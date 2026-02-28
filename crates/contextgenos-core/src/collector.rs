//! Collector trait and re-exports of shared data types.

use async_trait::async_trait;

// Re-export the data types from the store crate so the rest of the
// codebase can import from contextgenos_core::collector.
pub use contextgenos_store::{ContextItem, Sensitivity};

/// Health status returned by a collector's health check.
#[derive(Debug, Clone)]
pub struct CollectorHealth {
    pub healthy: bool,
    pub message: String,
}

/// Trait implemented by every context collector.
#[async_trait]
pub trait Collector: Send + Sync {
    /// Unique machine-readable collector name (e.g. "shell-history").
    fn name(&self) -> &str;

    /// Collect context items from the data source.
    ///
    /// Must NOT make network requests — local files and sockets only.
    async fn collect(&self) -> anyhow::Result<Vec<ContextItem>>;
}
