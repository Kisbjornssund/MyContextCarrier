//! Guardrails engine — safety checks for agent write-back operations.
//!
//! Evaluated before append_context and create_thread MCP tool calls.
//! Unlike privacy rules (which gate reads), guardrails gate writes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GuardrailTrigger {
    /// Fired when an agent calls append_context.
    WriteBack,
    /// Fired when an agent calls create_thread.
    ThreadCreate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GuardrailAction {
    /// Silently prevent the operation.
    Block,
    /// Remove the matched pattern from content before storing.
    Redact,
    /// Return a structured error — the caller must acknowledge before retrying.
    RequireConfirmation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guardrail {
    pub id: String,
    pub trigger: GuardrailTrigger,
    /// Substring or glob pattern matched against the content.
    pub pattern: String,
    pub action: GuardrailAction,
    /// Human-readable message returned when this guardrail fires.
    pub message: String,
}

pub struct GuardrailsEngine {
    rules: Vec<Guardrail>,
}

pub enum GuardrailDecision {
    Allow,
    Block { message: String },
    Redact { cleaned_content: String },
    RequireConfirmation { message: String },
}

impl GuardrailsEngine {
    pub fn new(rules: Vec<Guardrail>) -> Self {
        Self { rules }
    }

    pub fn from_config_file(path: &std::path::Path) -> anyhow::Result<Self> {
        #[derive(Deserialize)]
        struct Config {
            #[serde(default)]
            guardrails: Vec<Guardrail>,
        }
        if !path.exists() {
            return Ok(Self::new(vec![]));
        }
        let raw = std::fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&raw)?;
        Ok(Self::new(cfg.guardrails))
    }

    pub fn evaluate(
        &self,
        trigger: GuardrailTrigger,
        content: &str,
    ) -> GuardrailDecision {
        for rule in &self.rules {
            if rule.trigger != trigger {
                continue;
            }
            if !content_matches(&rule.pattern, content) {
                continue;
            }
            return match rule.action {
                GuardrailAction::Block => GuardrailDecision::Block {
                    message: rule.message.clone(),
                },
                GuardrailAction::Redact => {
                    let cleaned = content.replace(rule.pattern.as_str(), "[REDACTED]");
                    GuardrailDecision::Redact { cleaned_content: cleaned }
                }
                GuardrailAction::RequireConfirmation => {
                    GuardrailDecision::RequireConfirmation {
                        message: rule.message.clone(),
                    }
                }
            };
        }
        GuardrailDecision::Allow
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

fn content_matches(pattern: &str, content: &str) -> bool {
    let p = pattern.to_lowercase();
    let c = content.to_lowercase();
    if p == "*" { return true; }
    if p.contains('*') {
        // simple glob: split on * and check all parts appear in order
        let parts: Vec<&str> = p.split('*').collect();
        let mut pos = 0usize;
        for part in &parts {
            if part.is_empty() { continue; }
            if let Some(idx) = c[pos..].find(part.as_ref() as &str) {
                pos += idx + part.len();
            } else {
                return false;
            }
        }
        true
    } else {
        c.contains(p.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine(action: GuardrailAction) -> GuardrailsEngine {
        GuardrailsEngine::new(vec![Guardrail {
            id: "g1".into(),
            trigger: GuardrailTrigger::WriteBack,
            pattern: "secret".into(),
            action,
            message: "sensitive content detected".into(),
        }])
    }

    #[test]
    fn block_on_match() {
        let engine = make_engine(GuardrailAction::Block);
        let d = engine.evaluate(GuardrailTrigger::WriteBack, "this is a secret key");
        assert!(matches!(d, GuardrailDecision::Block { .. }));
    }

    #[test]
    fn allow_no_match() {
        let engine = make_engine(GuardrailAction::Block);
        let d = engine.evaluate(GuardrailTrigger::WriteBack, "this is fine");
        assert!(matches!(d, GuardrailDecision::Allow));
    }

    #[test]
    fn redact_replaces_content() {
        let engine = make_engine(GuardrailAction::Redact);
        let d = engine.evaluate(GuardrailTrigger::WriteBack, "my secret code");
        if let GuardrailDecision::Redact { cleaned_content } = d {
            assert!(cleaned_content.contains("[REDACTED]"));
            assert!(!cleaned_content.to_lowercase().contains("secret"));
        } else {
            panic!("expected Redact");
        }
    }

    #[test]
    fn wrong_trigger_is_ignored() {
        let engine = make_engine(GuardrailAction::Block);
        let d = engine.evaluate(GuardrailTrigger::ThreadCreate, "secret content");
        assert!(matches!(d, GuardrailDecision::Allow));
    }

    #[test]
    fn require_confirmation() {
        let engine = make_engine(GuardrailAction::RequireConfirmation);
        let d = engine.evaluate(GuardrailTrigger::WriteBack, "secret info");
        assert!(matches!(d, GuardrailDecision::RequireConfirmation { .. }));
    }
}
