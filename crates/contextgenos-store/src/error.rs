use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] duckdb::Error),

    #[error("Store not initialized. Run `contextgenos init` first.")]
    NotInitialized,

    #[error("Item not found: {id}")]
    NotFound { id: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
