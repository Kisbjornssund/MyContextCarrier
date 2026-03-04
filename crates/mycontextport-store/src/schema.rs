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
