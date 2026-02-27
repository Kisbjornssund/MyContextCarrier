use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Rule configuration error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
