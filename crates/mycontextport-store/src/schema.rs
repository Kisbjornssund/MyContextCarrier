//! Database schema definitions.

pub const CREATE_CONTEXT_ITEMS: &str = "
    CREATE TABLE IF NOT EXISTS context_items (
        id          TEXT PRIMARY KEY,
        content     TEXT NOT NULL,
        source      TEXT NOT NULL,
        collected_at BIGINT NOT NULL,
        url         TEXT,
        sensitivity TEXT NOT NULL DEFAULT 'unknown',
        metadata    JSON,
        embedding   BLOB
    );
";

pub const CREATE_INJECTION_LOG: &str = "
    CREATE TABLE IF NOT EXISTS injection_log (
        id          TEXT PRIMARY KEY,
        injected_at BIGINT NOT NULL,
        model       TEXT NOT NULL,
        items_used  JSON NOT NULL,
        rules_applied JSON NOT NULL,
        items_blocked JSON NOT NULL
    );
";

pub const CREATE_PRIVACY_RULES: &str = "
    CREATE TABLE IF NOT EXISTS privacy_rules (
        id          TEXT PRIMARY KEY,
        rule_type   TEXT NOT NULL,
        pattern     TEXT NOT NULL,
        action      TEXT NOT NULL,
        model_scope TEXT,
        created_at  BIGINT NOT NULL
    );
";

/// Migration: add thread_id column to context_items for v0.3 thread support.
/// DuckDB supports ADD COLUMN IF NOT EXISTS — safe to run on existing stores.
pub const MIGRATE_V2_THREAD_ID: &str =
    "ALTER TABLE context_items ADD COLUMN IF NOT EXISTS thread_id TEXT;";

pub const MIGRATE_V2_LOG_TRACE: &str = "
    ALTER TABLE injection_log ADD COLUMN IF NOT EXISTS thread_id TEXT;
    ALTER TABLE injection_log ADD COLUMN IF NOT EXISTS trace JSON;
";

pub const CREATE_COLLECTION_RUNS: &str = "
    CREATE TABLE IF NOT EXISTS collection_runs (
        id              TEXT PRIMARY KEY,
        collector       TEXT NOT NULL,
        started_at      BIGINT NOT NULL,
        finished_at     BIGINT,
        items_found     INTEGER,
        items_inserted  INTEGER,
        error           TEXT
    );
";

pub const CREATE_GRAPH_NODES: &str = "
    CREATE TABLE IF NOT EXISTS graph_nodes (
        id         TEXT PRIMARY KEY,
        label      TEXT NOT NULL,
        name       TEXT NOT NULL,
        metadata   JSON,
        created_at BIGINT NOT NULL
    );
";

pub const CREATE_GRAPH_EDGES: &str = "
    CREATE TABLE IF NOT EXISTS graph_edges (
        id         TEXT PRIMARY KEY,
        source_id  TEXT NOT NULL,
        target_id  TEXT NOT NULL,
        relation   TEXT NOT NULL,
        item_id    TEXT,
        created_at BIGINT NOT NULL
    );
";

pub const CREATE_THREADS: &str = "
    CREATE TABLE IF NOT EXISTS threads (
        id           TEXT PRIMARY KEY,
        name         TEXT NOT NULL,
        description  TEXT,
        created_at   BIGINT NOT NULL,
        updated_at   BIGINT NOT NULL,
        agent_scope  TEXT,
        instructions TEXT,
        archived     BOOLEAN NOT NULL DEFAULT FALSE
    );
";
