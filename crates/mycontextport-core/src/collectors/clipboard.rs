//! Clipboard collector — captures the current clipboard text.
//!
//! Uses `pbpaste` on macOS and `xclip`/`xsel` on Linux.
//! Skips collection silently if the clipboard is empty or unavailable.

use crate::collector::{Collector, ContextItem, Sensitivity};
use async_trait::async_trait;

pub struct ClipboardCollector;

impl ClipboardCollector {
    pub fn new() -> Self {
        Self
    }

    /// Read clipboard text using platform-specific CLI tools.
    fn read_clipboard() -> Option<String> {
        // macOS
        #[cfg(target_os = "macos")]
        {
            let out = std::process::Command::new("pbpaste").output().ok()?;
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if text.is_empty() { return None; }
            return Some(text);
        }

        // Linux — try xclip first, fall back to xsel
        #[cfg(target_os = "linux")]
        {
            if let Ok(out) = std::process::Command::new("xclip")
                .args(["-selection", "clipboard", "-o"])
                .output()
            {
                let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
            if let Ok(out) = std::process::Command::new("xsel")
                .args(["--clipboard", "--output"])
                .output()
            {
                let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
            return None;
        }

        // Unsupported platform
        #[allow(unreachable_code)]
        None
    }
}

impl Default for ClipboardCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Collector for ClipboardCollector {
    fn name(&self) -> &str {
        "clipboard"
    }

    async fn collect(&self) -> anyhow::Result<Vec<ContextItem>> {
        let Some(text) = Self::read_clipboard() else {
            return Ok(vec![]);
        };

        // Deduplicate by content: use a deterministic ID based on content hash
        // so re-collecting the same clipboard entry is a no-op.
        let id = format!("clipboard-{}", md5_hex(&text));
        let now = chrono::Utc::now().timestamp();

        Ok(vec![ContextItem {
            id,
            content: text,
            source: "clipboard".to_string(),
            collected_at: now,
            url: None,
            sensitivity: Sensitivity::Personal,
            metadata: serde_json::Value::Null,
        }])
    }
}

/// Minimal content-based ID: first 16 hex chars of a simple hash.
fn md5_hex(s: &str) -> String {
    // Use a simple FNV-1a hash — no crypto dependency needed for dedup IDs.
    let mut hash: u64 = 14695981039346656037;
    for byte in s.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}
