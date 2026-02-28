//! Local context store — DuckDB for structured data.
//!
//! All data is stored on the user's machine. Encryption is handled at the
//! store layer using keys that never leave the device.

pub mod duck;
pub mod error;
pub mod schema;
pub mod types;

pub use duck::ContextStore;
pub use error::Error;
pub use types::{ContextItem, Sensitivity};

pub type Result<T> = std::result::Result<T, Error>;
