//! The ContextGenOS daemon — drives periodic context collection.

use crate::collector::Collector;
use contextgenos_store::ContextStore;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

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

/// The daemon instance.
pub struct Daemon {
    pub config: DaemonConfig,
}

impl Daemon {
    pub fn new(config: DaemonConfig) -> Self {
        Self { config }
    }

    async fn run_collection(store: &Arc<ContextStore>, collectors: &[Box<dyn Collector>]) {
        for collector in collectors {
            match collector.collect().await {
                Ok(items) => {
                    let count = items.len();
                    match store.insert_items(&items) {
                        Ok(inserted) => {
                            info!(
                                collector = collector.name(),
                                collected = count,
                                inserted = inserted,
                                "Collection run complete"
                            );
                        }
                        Err(e) => {
                            error!(
                                collector = collector.name(),
                                error = %e,
                                "Failed to insert items into store"
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        collector = collector.name(),
                        error = %e,
                        "Collector returned an error"
                    );
                }
            }
        }
    }

    /// Start the collection loop. Runs until the future is dropped (e.g. via tokio::spawn).
    ///
    /// Runs collectors immediately on startup, then repeats every
    /// `config.collection_interval_secs` seconds.
    pub async fn run_loop(self, store: Arc<ContextStore>, collectors: Vec<Box<dyn Collector>>) {
        info!(
            store_path = ?self.config.store_path,
            interval_secs = self.config.collection_interval_secs,
            "Collection loop starting"
        );

        // Run immediately on startup
        Self::run_collection(&store, &collectors).await;

        // Then run on each interval tick
        let mut ticker = interval(Duration::from_secs(self.config.collection_interval_secs));
        ticker.tick().await; // consume the instant first tick
        loop {
            ticker.tick().await;
            Self::run_collection(&store, &collectors).await;
        }
    }
}
