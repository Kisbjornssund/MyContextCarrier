//! DuckDB-backed structured context store.

use crate::{schema, types::Sensitivity, CollectionRun, ContextItem, GraphEdge, GraphNode, InjectionLogEntry, Result, Thread};
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
        conn.execute_batch(schema::CREATE_GRAPH_NODES)?;
        conn.execute_batch(schema::CREATE_GRAPH_EDGES)?;
        conn.execute_batch(schema::CREATE_COLLECTION_RUNS)?;
        conn.execute_batch(schema::CREATE_THREADS)?;
        conn.execute_batch(schema::MIGRATE_V2_THREAD_ID)?;
        conn.execute_batch(schema::MIGRATE_V2_LOG_TRACE)?;
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

    /// Record an injection event in the audit log.
    pub fn log_injection(
        &self,
        model: &str,
        items_used: &[&str],
        items_blocked: &[&str],
        rules_applied: &[&str],
    ) -> Result<()> {
        self.log_injection_full(model, items_used, items_blocked, rules_applied, None, &[])
    }

    /// Record an injection event with optional thread_id and trace steps.
    pub fn log_injection_full(
        &self,
        model: &str,
        items_used: &[&str],
        items_blocked: &[&str],
        rules_applied: &[&str],
        thread_id: Option<&str>,
        trace: &[crate::TraceStep],
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let items_used_json = serde_json::to_string(items_used).map_err(anyhow::Error::from)?;
        let items_blocked_json = serde_json::to_string(items_blocked).map_err(anyhow::Error::from)?;
        let rules_json = serde_json::to_string(rules_applied).map_err(anyhow::Error::from)?;
        let trace_json = serde_json::to_string(trace).map_err(anyhow::Error::from)?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO injection_log \
             (id, injected_at, model, items_used, items_blocked, rules_applied, thread_id, trace) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![id, now, model, items_used_json, items_blocked_json, rules_json, thread_id, trace_json],
        )?;
        Ok(())
    }

    /// Return the most recent `limit` injection log entries ordered by time descending.
    pub fn query_log(&self, limit: usize) -> Result<Vec<InjectionLogEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, injected_at, model, items_used, items_blocked, rules_applied, thread_id, trace \
             FROM injection_log \
             ORDER BY injected_at DESC \
             LIMIT ?",
        )?;
        let entries = stmt
            .query_map(params![limit as i64], |row| {
                let items_used_str: String = row.get::<_, Option<String>>(3)?.unwrap_or_default();
                let items_blocked_str: String = row.get::<_, Option<String>>(4)?.unwrap_or_default();
                let rules_str: String = row.get::<_, Option<String>>(5)?.unwrap_or_default();
                let trace_str: String = row.get::<_, Option<String>>(7)?.unwrap_or_default();
                Ok(InjectionLogEntry {
                    id: row.get(0)?,
                    injected_at: row.get(1)?,
                    model: row.get(2)?,
                    items_used: serde_json::from_str(&items_used_str).unwrap_or_default(),
                    items_blocked: serde_json::from_str(&items_blocked_str).unwrap_or_default(),
                    rules_applied: serde_json::from_str(&rules_str).unwrap_or_default(),
                    thread_id: row.get(6)?,
                    trace: serde_json::from_str(&trace_str).unwrap_or_default(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(entries)
    }

    /// Return the most recent `limit` items from a specific source.
    pub fn query_by_source(&self, source: &str, limit: usize) -> Result<Vec<ContextItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, content, source, collected_at, url, sensitivity, metadata \
             FROM context_items \
             WHERE source = ? \
             ORDER BY collected_at DESC \
             LIMIT ?",
        )?;
        let items = stmt
            .query_map(params![source, limit as i64], |row| {
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

    /// Return item counts grouped by sensitivity level.
    pub fn count_by_sensitivity(&self) -> Result<std::collections::HashMap<String, i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT sensitivity, COUNT(*) FROM context_items GROUP BY sensitivity")?;
        let map = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(map)
    }

    /// Return item counts grouped by source collector.
    pub fn items_by_source(&self) -> Result<std::collections::HashMap<String, i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT source, COUNT(*) FROM context_items GROUP BY source")?;
        let map = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(map)
    }

    // ── Collection runs ──────────────────────────────────────────────────────

    pub fn start_run(&self, collector: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO collection_runs (id, collector, started_at) VALUES (?, ?, ?)",
            params![id, collector, now],
        )?;
        Ok(id)
    }

    pub fn finish_run(
        &self,
        run_id: &str,
        items_found: usize,
        items_inserted: usize,
        error: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE collection_runs SET finished_at=?, items_found=?, items_inserted=?, error=? WHERE id=?",
            params![now, items_found as i64, items_inserted as i64, error, run_id],
        )?;
        Ok(())
    }

    pub fn query_runs(&self, limit: usize) -> Result<Vec<CollectionRun>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, collector, started_at, finished_at, items_found, items_inserted, error \
             FROM collection_runs ORDER BY started_at DESC LIMIT ?",
        )?;
        let runs = stmt
            .query_map(params![limit as i64], |row| {
                Ok(CollectionRun {
                    id: row.get(0)?,
                    collector: row.get(1)?,
                    started_at: row.get(2)?,
                    finished_at: row.get(3)?,
                    items_found: row.get(4)?,
                    items_inserted: row.get(5)?,
                    error: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(runs)
    }

    // ── Memory threads ────────────────────────────────────────────────────────

    pub fn create_thread(
        &self,
        name: &str,
        description: Option<&str>,
        agent_scope: Option<&str>,
        instructions: Option<&str>,
    ) -> Result<Thread> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO threads (id, name, description, created_at, updated_at, agent_scope, instructions, archived) \
             VALUES (?, ?, ?, ?, ?, ?, ?, FALSE)",
            params![id, name, description, now, now, agent_scope, instructions],
        )?;
        Ok(Thread {
            id,
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
            agent_scope: agent_scope.map(|s| s.to_string()),
            instructions: instructions.map(|s| s.to_string()),
            archived: false,
        })
    }

    pub fn list_threads(&self, include_archived: bool) -> Result<Vec<Thread>> {
        let conn = self.conn.lock().unwrap();
        let sql = if include_archived {
            "SELECT id, name, description, created_at, updated_at, agent_scope, instructions, archived \
             FROM threads ORDER BY updated_at DESC"
        } else {
            "SELECT id, name, description, created_at, updated_at, agent_scope, instructions, archived \
             FROM threads WHERE archived = FALSE ORDER BY updated_at DESC"
        };
        let mut stmt = conn.prepare(sql)?;
        let threads = stmt
            .query_map([], |row| {
                Ok(Thread {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    agent_scope: row.get(5)?,
                    instructions: row.get(6)?,
                    archived: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(threads)
    }

    pub fn get_thread(&self, id: &str) -> Result<Option<Thread>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at, agent_scope, instructions, archived \
             FROM threads WHERE id = ?",
        )?;
        let thread = stmt
            .query_map(params![id], |row| {
                Ok(Thread {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    agent_scope: row.get(5)?,
                    instructions: row.get(6)?,
                    archived: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .next();
        Ok(thread)
    }

    pub fn archive_thread(&self, id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE threads SET archived = TRUE, updated_at = ? WHERE id = ?",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn assign_item_to_thread(&self, item_id: &str, thread_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE context_items SET thread_id = ? WHERE id = ?",
            params![thread_id, item_id],
        )?;
        conn.execute(
            "UPDATE threads SET updated_at = ? WHERE id = ?",
            params![now, thread_id],
        )?;
        Ok(())
    }

    pub fn query_thread_items(&self, thread_id: &str, limit: usize) -> Result<Vec<ContextItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, content, source, collected_at, url, sensitivity, metadata \
             FROM context_items WHERE thread_id = ? \
             ORDER BY collected_at DESC LIMIT ?",
        )?;
        let items = stmt
            .query_map(params![thread_id, limit as i64], |row| {
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

    // ── Knowledge graph ───────────────────────────────────────────────────────

    /// Insert a node, ignoring conflicts on id (upsert by name+label instead).
    pub fn upsert_graph_node(&self, label: &str, name: &str, metadata: &serde_json::Value) -> Result<String> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        // Find existing node with same label+name
        let existing: Option<String> = {
            let mut stmt = conn.prepare(
                "SELECT id FROM graph_nodes WHERE label = ? AND name = ? LIMIT 1",
            )?;
            stmt.query_map(params![label, name], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .next()
        };
        if let Some(id) = existing {
            return Ok(id);
        }
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO graph_nodes (id, label, name, metadata, created_at) VALUES (?, ?, ?, ?, ?)",
            params![id, label, name, metadata.to_string(), now],
        )?;
        Ok(id)
    }

    pub fn upsert_graph_edge(
        &self,
        source_id: &str,
        target_id: &str,
        relation: &str,
        item_id: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        // Skip if edge already exists
        let exists: bool = {
            let mut stmt = conn.prepare(
                "SELECT 1 FROM graph_edges WHERE source_id=? AND target_id=? AND relation=? LIMIT 1",
            )?;
            stmt.query_map(params![source_id, target_id, relation], |_| Ok(()))?
                .next()
                .is_some()
        };
        if exists {
            return Ok(());
        }
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO graph_edges (id, source_id, target_id, relation, item_id, created_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
            params![id, source_id, target_id, relation, item_id, now],
        )?;
        Ok(())
    }

    /// Find nodes whose name contains `query` (case-insensitive).
    pub fn search_graph_nodes(&self, query: &str) -> Result<Vec<GraphNode>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query.to_lowercase());
        let mut stmt = conn.prepare(
            "SELECT id, label, name, metadata, created_at FROM graph_nodes \
             WHERE LOWER(name) LIKE ? ORDER BY name LIMIT 50",
        )?;
        let nodes = stmt
            .query_map(params![pattern], |row| {
                let metadata_str: String = row.get::<_, Option<String>>(3)?.unwrap_or_default();
                Ok(GraphNode {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    name: row.get(2)?,
                    metadata: serde_json::from_str(&metadata_str).unwrap_or(serde_json::Value::Null),
                    created_at: row.get(4)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(nodes)
    }

    /// Return context items linked to a graph node via graph_edges.
    pub fn items_for_node(&self, node_id: &str, limit: usize) -> Result<Vec<ContextItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ci.id, ci.content, ci.source, ci.collected_at, ci.url, ci.sensitivity, ci.metadata \
             FROM context_items ci \
             JOIN graph_edges ge ON ge.item_id = ci.id \
             WHERE ge.source_id = ? OR ge.target_id = ? \
             ORDER BY ci.collected_at DESC LIMIT ?",
        )?;
        let items = stmt
            .query_map(params![node_id, node_id, limit as i64], |row| {
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

    pub fn graph_node_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM graph_nodes")?;
        Ok(stmt.query_row([], |row| row.get(0))?)
    }

    pub fn graph_edge_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM graph_edges")?;
        Ok(stmt.query_row([], |row| row.get(0))?)
    }
}
