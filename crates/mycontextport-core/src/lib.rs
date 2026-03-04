//! MyContextPort core daemon.
//!
//! Orchestrates context collection from data sources, storage in the local
//! context store, and injection into AI tools via the MCP server.

pub mod collector;
pub mod collectors;
pub mod daemon;
pub mod error;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
