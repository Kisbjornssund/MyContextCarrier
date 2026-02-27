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
