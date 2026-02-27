//! Collector registry — manages active context collectors.

use anyhow::Result;
use serde::{Deserialize, Serialize};

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

/// Health status returned by a collector's health check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorHealth {
    pub healthy: bool,
    pub message: String,
}
