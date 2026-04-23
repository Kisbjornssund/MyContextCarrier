//! Subprocess bridge that runs Python collectors and deserialises their output.

use crate::collector::{Collector, CollectorHealth};
use anyhow::{Context, Result};
use async_trait::async_trait;
use mycontextport_store::{ContextItem, Sensitivity};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::warn;

/// A wrapper that invokes a Python collector script as a subprocess.
///
/// Protocol:
///   stdin  → JSON config object
///   stdout → JSON array of ContextItem dicts (see sdk/python/mycontextport/types.py)
///   `--health` flag → `{"healthy": bool, "message": string}`
pub struct PythonCollector {
    pub name: String,
    pub script_path: PathBuf,
    pub config: serde_json::Value,
    /// Per-run timeout in seconds. Subprocess is killed if it exceeds this.
    pub timeout_secs: u64,
}

impl PythonCollector {
    pub fn new(
        name: impl Into<String>,
        script_path: impl Into<PathBuf>,
        config: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            script_path: script_path.into(),
            config,
            timeout_secs: 60,
        }
    }

    async fn run_subprocess(&self, extra_arg: Option<&str>) -> Result<Vec<u8>> {
        let config_json = serde_json::to_string(&self.config)?;
        let mut cmd = Command::new("python3");
        cmd.arg(&self.script_path)
            .arg("--config")
            .arg(&config_json)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(arg) = extra_arg {
            cmd.arg(arg);
        }

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            cmd.output(),
        )
        .await
        .context("collector subprocess timed out")?
        .context("failed to spawn collector subprocess")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(
                collector = %self.name,
                exit_code = ?output.status.code(),
                stderr = %stderr,
                "Collector subprocess exited with error"
            );
        }

        Ok(output.stdout)
    }
}

/// Wire format matching `ContextItem.to_dict()` from the Python SDK.
#[derive(Deserialize)]
struct RawItem {
    id: String,
    content: String,
    source: String,
    collected_at: i64,
    url: Option<String>,
    sensitivity: Option<String>,
    #[serde(default)]
    metadata: serde_json::Value,
}

fn parse_sensitivity(s: Option<&str>) -> Sensitivity {
    match s {
        Some("personal") => Sensitivity::Personal,
        Some("work") => Sensitivity::Work,
        Some("health") => Sensitivity::Health,
        Some("financial") => Sensitivity::Financial,
        Some("public") => Sensitivity::Public,
        _ => Sensitivity::Unknown,
    }
}

#[async_trait]
impl Collector for PythonCollector {
    fn name(&self) -> &str {
        &self.name
    }

    async fn collect(&self) -> Result<Vec<ContextItem>> {
        let stdout = match self.run_subprocess(None).await {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!(collector = %self.name, error = %e, "Subprocess failed");
                return Ok(vec![]);
            }
        };

        if stdout.is_empty() {
            return Ok(vec![]);
        }

        let raw: Vec<RawItem> = match serde_json::from_slice(&stdout) {
            Ok(v) => v,
            Err(e) => {
                warn!(
                    collector = %self.name,
                    error = %e,
                    "Failed to parse collector output"
                );
                return Ok(vec![]);
            }
        };

        let items = raw
            .into_iter()
            .map(|r| ContextItem {
                id: r.id,
                content: r.content,
                source: r.source,
                collected_at: r.collected_at,
                url: r.url,
                sensitivity: parse_sensitivity(r.sensitivity.as_deref()),
                metadata: r.metadata,
            })
            .collect();

        Ok(items)
    }

    async fn health_check(&self) -> CollectorHealth {
        #[derive(Deserialize)]
        struct HealthResult {
            healthy: bool,
            message: String,
        }

        match self.run_subprocess(Some("--health")).await {
            Ok(bytes) if !bytes.is_empty() => {
                match serde_json::from_slice::<HealthResult>(&bytes) {
                    Ok(h) => CollectorHealth { healthy: h.healthy, message: h.message },
                    Err(_) => CollectorHealth {
                        healthy: false,
                        message: "Invalid health check response from collector".into(),
                    },
                }
            }
            _ => CollectorHealth {
                healthy: false,
                message: format!("Collector script not found or failed: {:?}", self.script_path),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn trivial_collector_script() -> NamedTempFile {
        let mut f = tempfile::Builder::new().suffix(".py").tempfile().unwrap();
        write!(
            f,
            r#"
import sys, json, time

config = json.loads(sys.argv[2]) if "--config" in sys.argv else {{}}

if "--health" in sys.argv:
    print(json.dumps({{"healthy": True, "message": "ok"}}))
    sys.exit(0)

items = [{{
    "id": "test-001",
    "content": "hello from python",
    "source": "test",
    "collected_at": int(time.time()),
    "url": None,
    "sensitivity": "personal",
    "metadata": {{}}
}}]
print(json.dumps(items))
"#
        )
        .unwrap();
        f
    }

    #[tokio::test]
    async fn collect_parses_output() {
        let script = trivial_collector_script();
        let collector = PythonCollector::new("test", script.path(), serde_json::json!({}));
        let items = collector.collect().await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content, "hello from python");
        assert_eq!(items[0].id, "test-001");
    }

    #[tokio::test]
    async fn health_check_parses_response() {
        let script = trivial_collector_script();
        let collector = PythonCollector::new("test", script.path(), serde_json::json!({}));
        let health = collector.health_check().await;
        assert!(health.healthy);
        assert_eq!(health.message, "ok");
    }

    #[tokio::test]
    async fn collect_returns_empty_on_bad_script() {
        let collector =
            PythonCollector::new("bad", PathBuf::from("/nonexistent/script.py"), serde_json::json!({}));
        let items = collector.collect().await.unwrap();
        assert!(items.is_empty());
    }
}
