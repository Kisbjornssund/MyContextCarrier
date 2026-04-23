//! Rule-based entity extraction from context item text.
//!
//! Recognises:
//!   - People: @mention patterns and "Name Surname" in calendar/email metadata
//!   - Tools: a built-in list of common dev/AI tools
//!   - Projects: headings from notes (# Title) and workspace names from VSCode
//!   - Concepts: top-level noun phrases from shell commands (heuristic)

use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityKind {
    Person,
    Project,
    Tool,
    Concept,
}

impl EntityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityKind::Person => "person",
            EntityKind::Project => "project",
            EntityKind::Tool => "tool",
            EntityKind::Concept => "concept",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Entity {
    pub kind: EntityKind,
    pub name: String,
}

/// Known tools matched by exact name (case-insensitive).
static KNOWN_TOOLS: &[&str] = &[
    "git", "cargo", "npm", "yarn", "pnpm", "docker", "kubectl", "terraform",
    "ansible", "make", "cmake", "gradle", "maven", "pip", "poetry", "uv",
    "rustc", "clang", "gcc", "python", "node", "deno", "bun", "go", "java",
    "vim", "nvim", "emacs", "vscode", "cursor", "zed",
    "claude", "openai", "ollama", "llamafile",
    "postgres", "mysql", "sqlite", "duckdb", "redis", "mongodb",
    "nginx", "caddy", "traefik",
    "github", "gitlab", "bitbucket", "jira", "linear", "notion", "obsidian",
    "slack", "discord", "teams",
    "aws", "gcp", "azure", "fly", "railway", "vercel", "netlify",
];

pub fn extract_entities(source: &str, content: &str, metadata: &serde_json::Value) -> Vec<Entity> {
    let mut entities: HashSet<Entity> = HashSet::new();

    // @mention → person
    let mention_re = Regex::new(r"@([A-Za-z][A-Za-z0-9._-]{1,39})").unwrap();
    for cap in mention_re.captures_iter(content) {
        entities.insert(Entity { kind: EntityKind::Person, name: cap[1].to_string() });
    }

    // Markdown heading → project
    let heading_re = Regex::new(r"(?m)^#\s+(.+)$").unwrap();
    for cap in heading_re.captures_iter(content) {
        let name = cap[1].trim().to_string();
        if name.len() >= 3 && name.len() <= 60 {
            entities.insert(Entity { kind: EntityKind::Project, name });
        }
    }

    // VSCode workspace names
    if source == "vscode" {
        if let Some(serde_json::Value::String(ws)) = metadata.get("uri") {
            if let Some(name) = ws.split('/').last() {
                let clean = name.replace(".code-workspace", "");
                if !clean.is_empty() {
                    entities.insert(Entity { kind: EntityKind::Project, name: clean });
                }
            }
        }
    }

    // Calendar event summaries → project (if it looks like a project meeting)
    if source == "calendar" {
        let project_re = Regex::new(r"\b([A-Z][a-z]+(?:\s[A-Z][a-z]+)*)\b").unwrap();
        for cap in project_re.captures_iter(content) {
            let name = cap[1].to_string();
            if name.split_whitespace().count() >= 2 && name.len() <= 40 {
                entities.insert(Entity { kind: EntityKind::Concept, name });
            }
        }
    }

    // Known tool names in any content
    let content_lower = content.to_lowercase();
    for &tool in KNOWN_TOOLS {
        // Match as a whole word
        let word_re = Regex::new(&format!(r"\b{}\b", regex::escape(tool))).unwrap();
        if word_re.is_match(&content_lower) {
            entities.insert(Entity { kind: EntityKind::Tool, name: tool.to_string() });
        }
    }

    // Email sender names → person
    if source == "email" {
        if let Some(serde_json::Value::String(name)) = metadata.get("sender_name") {
            let clean = name.trim().to_string();
            if clean.len() >= 2 && !clean.contains('@') {
                entities.insert(Entity { kind: EntityKind::Person, name: clean });
            }
        }
    }

    entities.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_mentions() {
        let entities = extract_entities("notes", "Hey @alice and @bob, check this out", &json!({}));
        let names: Vec<_> = entities.iter().filter(|e| e.kind == EntityKind::Person).map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"alice"));
        assert!(names.contains(&"bob"));
    }

    #[test]
    fn extracts_tools() {
        let entities = extract_entities("shell-history", "cargo build --release && docker push", &json!({}));
        let tools: Vec<_> = entities.iter().filter(|e| e.kind == EntityKind::Tool).map(|e| e.name.as_str()).collect();
        assert!(tools.contains(&"cargo"));
        assert!(tools.contains(&"docker"));
    }

    #[test]
    fn extracts_heading_as_project() {
        let entities = extract_entities("notes", "# MyContextPort\n\nSome notes here.", &json!({}));
        let projects: Vec<_> = entities.iter().filter(|e| e.kind == EntityKind::Project).map(|e| e.name.as_str()).collect();
        assert!(projects.contains(&"MyContextPort"));
    }

    #[test]
    fn extracts_vscode_workspace() {
        let entities = extract_entities("vscode", "VSCode workspace: my-project", &json!({"uri": "file:///Users/dev/my-project"}));
        let projects: Vec<_> = entities.iter().filter(|e| e.kind == EntityKind::Project).map(|e| e.name.as_str()).collect();
        assert!(projects.contains(&"my-project"));
    }

    #[test]
    fn extracts_email_sender() {
        let entities = extract_entities("email", "Meeting notes", &json!({"sender_name": "Alice Smith"}));
        let people: Vec<_> = entities.iter().filter(|e| e.kind == EntityKind::Person).map(|e| e.name.as_str()).collect();
        assert!(people.contains(&"Alice Smith"));
    }
}
