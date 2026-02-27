//! DuckDB-backed structured context store.

use crate::Result;
use std::path::Path;

/// The structured context store backed by an embedded DuckDB database.
pub struct ContextStore {
    // TODO: connection pool
}

impl ContextStore {
    /// Open or create a context store at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let _ = path;
        // TODO: open DuckDB at path
        // TODO: run schema migrations
        Ok(Self {})
    }

    /// Initialize schema on a fresh database.
    pub fn initialize(&self) -> Result<()> {
        // TODO: CREATE TABLE context_items (...)
        // TODO: CREATE TABLE injection_log (...)
        // TODO: CREATE TABLE privacy_rules (...)
        Ok(())
    }
}
