//! The privacy rules engine — evaluates rules against context items.

use crate::rule::{PrivacyRule, RuleAction};
use crate::Result;

/// Decision returned by the engine for a context item.
#[derive(Debug, Clone)]
pub struct InjectionDecision {
    pub action: RuleAction,
    pub rules_matched: Vec<String>,
}

/// Evaluates privacy rules against context items before injection.
pub struct PrivacyEngine {
    #[allow(dead_code)] // used once evaluate() is implemented
    rules: Vec<PrivacyRule>,
}

impl PrivacyEngine {
    pub fn new(rules: Vec<PrivacyRule>) -> Self {
        Self { rules }
    }

    pub fn from_config_file(path: &std::path::Path) -> Result<Self> {
        let _ = path;
        // TODO: parse YAML/TOML privacy rules config
        Ok(Self::new(vec![]))
    }

    /// Evaluate rules for a given context item and target model.
    pub fn evaluate(
        &self,
        item_sensitivity: &str,
        item_source: &str,
        item_url: Option<&str>,
        target_model: &str,
    ) -> InjectionDecision {
        let _ = (item_sensitivity, item_source, item_url, target_model);
        // TODO: evaluate each rule in order, return first matching action
        InjectionDecision {
            action: RuleAction::Allow,
            rules_matched: vec![],
        }
    }
}
