//! Core data types shared across the store boundary.

use serde::{Deserialize, Serialize};

/// Sensitivity classification for privacy rules enforcement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Sensitivity {
    #[default]
    Unknown,
    Public,
    Work,
    Personal,
    Health,
    Financial,
}

impl Sensitivity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Sensitivity::Unknown => "unknown",
            Sensitivity::Public => "public",
            Sensitivity::Work => "work",
            Sensitivity::Personal => "personal",
            Sensitivity::Health => "health",
            Sensitivity::Financial => "financial",
        }
    }
}

impl std::str::FromStr for Sensitivity {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "public" => Ok(Sensitivity::Public),
            "work" => Ok(Sensitivity::Work),
            "personal" => Ok(Sensitivity::Personal),
            "health" => Ok(Sensitivity::Health),
            "financial" => Ok(Sensitivity::Financial),
            _ => Ok(Sensitivity::Unknown),
        }
    }
}

/// A node in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    /// Category: "person", "project", "tool", "concept"
    pub label: String,
    pub name: String,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

/// A directed edge between two graph nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    /// e.g. "mentioned_with", "used_in", "worked_on"
    pub relation: String,
    /// The context item that produced this edge.
    pub item_id: Option<String>,
    pub created_at: i64,
}

/// A record of a single collector run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRun {
    pub id: String,
    pub collector: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub items_found: Option<i64>,
    pub items_inserted: Option<i64>,
    pub error: Option<String>,
}

/// A project-scoped memory thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    /// Glob pattern restricting which MCP client can access this thread.
    pub agent_scope: Option<String>,
    /// Custom instructions injected alongside thread context.
    pub instructions: Option<String>,
    pub archived: bool,
}

/// A single step in the injection trace (chain-of-thought for each MCP call).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub step: String,
    pub duration_ms: u64,
    pub detail: serde_json::Value,
}

/// An entry in the injection audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionLogEntry {
    pub id: String,
    pub injected_at: i64,
    pub model: String,
    pub items_used: Vec<String>,
    pub items_blocked: Vec<String>,
    pub rules_applied: Vec<String>,
    pub thread_id: Option<String>,
    #[serde(default)]
    pub trace: Vec<TraceStep>,
}

/// A context item produced by a collector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    /// Unique identifier for this item.
    pub id: String,
    /// The raw text content.
    pub content: String,
    /// Which collector produced this item.
    pub source: String,
    /// When this item was collected (Unix timestamp).
    pub collected_at: i64,
    /// Optional URL or file path associated with this item.
    pub url: Option<String>,
    /// Sensitivity classification.
    pub sensitivity: Sensitivity,
    /// Arbitrary metadata from the collector.
    pub metadata: serde_json::Value,
}
