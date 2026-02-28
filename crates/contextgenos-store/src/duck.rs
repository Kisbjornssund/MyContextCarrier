//! DuckDB-backed structured context store.

use crate::{schema, types::Sensitivity, ContextItem, Result};
use duckdb::{params, Connection};
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

/// The structured context store backed by an embedded DuckDB database.
pub struct ContextStore {
    conn: Mutex<Connection>,
}

impl ContextStore {
    /// Open or create a context store. The database file is placed at
    /// `dir/context.db`; `dir` is created if it does not exist.
    pub fn open(dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(dir)?;
        let db_path = dir.join("context.db");
        let conn = Connection::open(&db_path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Initialize schema. Safe to call on an already-initialized store
    /// because all statements use `CREATE TABLE IF NOT EXISTS`.
    pub fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(schema::CREATE_CONTEXT_ITEMS)?;
        conn.execute_batch(schema::CREATE_INJECTION_LOG)?;
        conn.execute_batch(schema::CREATE_PRIVACY_RULES)?;
        Ok(())
    }

    /// Insert items into the store, ignoring duplicates by `id`.
    pub fn insert_items(&self, items: &[ContextItem]) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut inserted = 0usize;
        for item in items {
            let sql = "INSERT INTO context_items \
                (id, content, source, collected_at, url, sensitivity, metadata) \
                VALUES (?, ?, ?, ?, ?, ?, ?) \
                ON CONFLICT (id) DO NOTHING";
            let rows = conn.execute(
                sql,
                params![
                    item.id,
                    item.content,
                    item.source,
                    item.collected_at,
                    item.url,
                    item.sensitivity.as_str(),
                    item.metadata.to_string(),
                ],
            )?;
            inserted += rows;
        }
        Ok(inserted)
    }

    /// Return the most recent `limit` items ordered by collection time descending.
    pub fn query_recent(&self, limit: usize) -> Result<Vec<ContextItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, content, source, collected_at, url, sensitivity, metadata \
             FROM context_items \
             ORDER BY collected_at DESC \
             LIMIT ?",
        )?;
        let items = stmt
            .query_map(params![limit as i64], |row| {
                let sensitivity_str: String = row.get(5)?;
                let metadata_str: String = row.get::<_, Option<String>>(6)?.unwrap_or_default();
                Ok(ContextItem {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    source: row.get(2)?,
                    collected_at: row.get(3)?,
                    url: row.get(4)?,
                    sensitivity: Sensitivity::from_str(&sensitivity_str)
                        .unwrap_or(Sensitivity::Unknown),
                    metadata: serde_json::from_str(&metadata_str)
                        .unwrap_or(serde_json::Value::Null),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    }

    /// Return the total number of context items stored.
    pub fn count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM context_items")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }
}
