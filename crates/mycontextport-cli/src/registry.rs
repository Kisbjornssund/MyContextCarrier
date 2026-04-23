//! Built-in collector registry and collectors.toml config loader.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Metadata for a built-in Python collector.
#[allow(dead_code)]
pub struct CollectorSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub platforms: &'static [&'static str],
}

/// All collectors that ship with MyContextPort.
pub fn builtin_registry() -> HashMap<&'static str, CollectorSpec> {
    let mut m = HashMap::new();
    m.insert(
        "calendar",
        CollectorSpec {
            name: "calendar",
            description: "Upcoming and recent calendar events from local iCal files",
            platforms: &["macos", "linux", "windows"],
        },
    );
    m.insert(
        "browser",
        CollectorSpec {
            name: "browser",
            description: "Recent browser history from Chrome and Firefox",
            platforms: &["macos", "linux"],
        },
    );
    m.insert(
        "notes",
        CollectorSpec {
            name: "notes",
            description: "Markdown notes and Obsidian vault files",
            platforms: &["macos", "linux", "windows"],
        },
    );
    m.insert(
        "email",
        CollectorSpec {
            name: "email",
            description: "Recent email metadata from local mbox files",
            platforms: &["macos", "linux", "windows"],
        },
    );
    m.insert(
        "vscode",
        CollectorSpec {
            name: "vscode",
            description: "Recent VSCode workspaces and edited files",
            platforms: &["macos", "linux", "windows"],
        },
    );
    m
}

/// A Python collector entry from collectors.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonCollectorEntry {
    pub name: String,
    pub script: String,
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
    #[serde(default)]
    pub config: serde_json::Value,
}

fn default_interval() -> u64 {
    900
}

/// The full collectors.toml structure.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CollectorsConfig {
    #[serde(default, rename = "python")]
    pub python: Vec<PythonCollectorEntry>,
}

impl CollectorsConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&raw)?)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

pub fn collectors_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".mycontextport")
        .join("collectors.toml")
}

/// Expand `~` in a path string.
pub fn expand_path(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/") {
        dirs::home_dir().unwrap_or_default().join(rest)
    } else {
        PathBuf::from(s)
    }
}
