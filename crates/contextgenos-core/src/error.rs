use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Store error: {0}")]
    Store(#[from] contextgenos_store::Error),

    #[error("Privacy rules error: {0}")]
    Privacy(#[from] contextgenos_privacy::Error),

    #[error("Collector error: {source}")]
    Collector {
        collector: String,
        #[source]
        source: anyhow::Error,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
