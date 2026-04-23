//! Per-collector scheduler.
//!
//! Each collector runs in its own tokio task on an independent interval,
//! so a slow collector never blocks the others. Runs are logged to the
//! collection_runs table for dashboard visibility.

use crate::collector::Collector;
use mycontextport_store::ContextStore;
use std::sync::Arc;
use tokio::time::{interval, timeout, Duration};
use tracing::{error, info, warn};

pub struct CollectorSchedule {
    pub collector: Box<dyn Collector>,
    /// How often to run this collector in seconds.
    pub interval_secs: u64,
    /// Per-run hard timeout in seconds. Default: 60.
    pub timeout_secs: u64,
}

impl CollectorSchedule {
    pub fn new(collector: Box<dyn Collector>, interval_secs: u64) -> Self {
        Self { collector, interval_secs, timeout_secs: 60 }
    }
}

pub struct Scheduler {
    schedules: Vec<CollectorSchedule>,
    store: Arc<ContextStore>,
}

impl Scheduler {
    pub fn new(store: Arc<ContextStore>, schedules: Vec<CollectorSchedule>) -> Self {
        Self { schedules, store }
    }

    /// Start the scheduler. Each collector runs concurrently in its own tokio
    /// task. Blocks until all tasks complete (they run forever).
    pub async fn run(self) {
        info!(collectors = self.schedules.len(), "Scheduler starting");

        let mut handles = vec![];

        for schedule in self.schedules {
            let store = Arc::clone(&self.store);
            let interval_secs = schedule.interval_secs;
            let timeout_secs = schedule.timeout_secs;
            let collector = schedule.collector;

            let handle = tokio::spawn(async move {
                let name = collector.name().to_string();
                info!(collector = %name, interval_secs, "Collector scheduled");

                // Run immediately on startup, then on each interval tick.
                run_once(&store, &*collector, timeout_secs).await;

                let mut ticker = interval(Duration::from_secs(interval_secs));
                ticker.tick().await; // consume the instant first tick
                loop {
                    ticker.tick().await;
                    run_once(&store, &*collector, timeout_secs).await;
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }
    }
}

async fn run_once(store: &Arc<ContextStore>, collector: &dyn Collector, timeout_secs: u64) {
    let name = collector.name().to_string();

    let run_id = match store.start_run(&name) {
        Ok(id) => id,
        Err(e) => {
            warn!(collector = %name, error = %e, "Failed to start run log");
            String::new()
        }
    };

    let result = timeout(Duration::from_secs(timeout_secs), collector.collect()).await;

    match result {
        Err(_elapsed) => {
            warn!(collector = %name, timeout_secs, "Collector timed out");
            if !run_id.is_empty() {
                let _ = store.finish_run(&run_id, 0, 0, Some("timed out"));
            }
        }
        Ok(Err(e)) => {
            warn!(collector = %name, error = %e, "Collector error");
            if !run_id.is_empty() {
                let _ = store.finish_run(&run_id, 0, 0, Some(&e.to_string()));
            }
        }
        Ok(Ok(items)) => {
            let found = items.len();
            match store.insert_items(&items) {
                Ok(inserted) => {
                    info!(collector = %name, found, inserted, "Collection run complete");
                    if !run_id.is_empty() {
                        let _ = store.finish_run(&run_id, found, inserted, None);
                    }
                }
                Err(e) => {
                    error!(collector = %name, error = %e, "Failed to insert items");
                    if !run_id.is_empty() {
                        let _ = store.finish_run(&run_id, found, 0, Some(&e.to_string()));
                    }
                }
            }
        }
    }
}
