//! Local context store — DuckDB for structured data, Qdrant for vector search.
//!
//! All data is stored on the user's machine. Encryption is handled at the
//! store layer using keys that never leave the device.

pub mod duck;
pub mod error;
pub mod schema;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
