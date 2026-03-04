//! Privacy rule definitions.

use serde::{Deserialize, Serialize};

/// A single privacy rule configured by the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyRule {
    pub id: String,
    pub rule_type: RuleType,
    pub pattern: String,
    pub action: RuleAction,
    /// If set, this rule only applies when injecting into this model.
    pub model_scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    /// Match on the URL or file path of the context item.
    UrlPattern,
    /// Match on the sensitivity classification.
    Sensitivity,
    /// Match on the source collector name.
    Source,
    /// Match on text content using a pattern.
    ContentPattern,
    /// Match on the backend type ("cloud" or "local"). Used by the agent
    /// layer to select inference backend; evaluated separately from item
    /// injection rules.
    BackendScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    /// Allow injection (default).
    Allow,
    /// Block injection for this item.
    Block,
    /// Inject only a summary, not full content.
    Summarize,
}

/// What to do when no rule matches and the connecting client's name does not
/// appear in any `model_scope` across the entire ruleset.
///
/// Set via `[defaults] unknown_client = "deny"` in `privacy.toml`.
/// Defaults to `Allow` so the server works without any config file.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UnknownClientPolicy {
    /// Unrecognised clients are treated the same as known ones: allow
    /// anything not explicitly blocked. Preserves existing allow-all
    /// behaviour when no config file is present.
    #[default]
    Allow,
    /// Block all context for clients whose name does not match any rule's
    /// `model_scope`. Prevents a rogue or misconfigured client from
    /// bypassing rules by misrepresenting its identity.
    Deny,
}
