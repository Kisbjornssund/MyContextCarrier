//! Shell history collector — reads ~/.zsh_history or ~/.bash_history.

use crate::collector::{Collector, ContextItem, Sensitivity};
use async_trait::async_trait;
use std::collections::HashSet;
use std::path::PathBuf;
use uuid::Uuid;

pub struct ShellHistoryCollector;

impl ShellHistoryCollector {
    pub fn new() -> Self {
        Self
    }

    fn find_history_file() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        let zsh = home.join(".zsh_history");
        if zsh.exists() {
            return Some(zsh);
        }
        let bash = home.join(".bash_history");
        if bash.exists() {
            return Some(bash);
        }
        None
    }

    /// Parse a single history file line into a command string, or None if blank.
    ///
    /// Zsh extended history format: `: <timestamp>:<elapsed>;<command>`
    /// Plain format (bash and simple zsh): just the command text.
    fn parse_line(line: &str) -> Option<&str> {
        if let Some(rest) = line.strip_prefix(": ") {
            // Extended format — find the semicolon separator
            if let Some(semicolon) = rest.find(';') {
                let cmd = &rest[semicolon + 1..];
                let trimmed = cmd.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed);
                }
            }
            return None;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    }
}

impl Default for ShellHistoryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Collector for ShellHistoryCollector {
    fn name(&self) -> &str {
        "shell-history"
    }

    async fn collect(&self) -> anyhow::Result<Vec<ContextItem>> {
        let Some(path) = Self::find_history_file() else {
            tracing::warn!("No shell history file found (~/.zsh_history or ~/.bash_history)");
            return Ok(vec![]);
        };

        // Use from_utf8_lossy because zsh history can contain non-UTF-8 bytes
        let raw = std::fs::read(&path)?;
        let content = String::from_utf8_lossy(&raw);
        let now = chrono::Utc::now().timestamp();

        // Collect the last 200 unique commands in chronological order.
        // Strategy: iterate from end, dedup via HashSet, take 200, reverse back.
        let all_commands: Vec<String> = content
            .lines()
            .filter_map(Self::parse_line)
            .map(str::to_string)
            .collect();

        let mut seen: HashSet<String> = HashSet::new();
        let unique: Vec<String> = all_commands
            .into_iter()
            .rev()
            .filter(|cmd| seen.insert(cmd.clone()))
            .take(200)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        let items = unique
            .into_iter()
            .map(|cmd| ContextItem {
                id: Uuid::new_v4().to_string(),
                content: cmd,
                source: "shell-history".to_string(),
                collected_at: now,
                url: None,
                sensitivity: Sensitivity::Work,
                metadata: serde_json::Value::Null,
            })
            .collect();

        Ok(items)
    }
}
