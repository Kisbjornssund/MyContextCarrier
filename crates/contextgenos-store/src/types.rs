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
