//! The ContextGenOS daemon — background process that drives context collection.

use tracing::info;

/// Daemon configuration.
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Path to the context store directory.
    pub store_path: std::path::PathBuf,
    /// Collection interval in seconds.
    pub collection_interval_secs: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            // data_local_dir: ~/Library/Application Support on macOS,
            // ~/.local/share on Linux, %LOCALAPPDATA% on Windows.
            store_path: dirs::data_local_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
                .join("ContextGenOS"),
            collection_interval_secs: 900, // 15 minutes
        }
    }
}

/// The running daemon instance.
pub struct Daemon {
    config: DaemonConfig,
}

impl Daemon {
    pub fn new(config: DaemonConfig) -> Self {
        Self { config }
    }

    /// Start the daemon. Runs until the process is terminated.
    pub async fn run(&self) -> crate::Result<()> {
        info!(
            store_path = ?self.config.store_path,
            interval_secs = self.config.collection_interval_secs,
            "ContextGenOS daemon starting"
        );

        // TODO: initialize store
        // TODO: load and start collectors
        // TODO: start collection loop
        // TODO: start MCP server

        info!("ContextGenOS daemon ready");
        Ok(())
    }
}
