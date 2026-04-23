//! The privacy rules engine — evaluates rules against context items.

use crate::rule::{PrivacyRule, RuleAction, RuleType, UnknownClientPolicy};
use crate::Result;

/// Decision returned by the engine for a context item.
#[derive(Debug, Clone)]
pub struct InjectionDecision {
    pub action: RuleAction,
    pub rules_matched: Vec<String>,
}

/// Evaluates privacy rules against context items before injection.
pub struct PrivacyEngine {
    rules: Vec<PrivacyRule>,
    /// Fallback for clients whose name matches no `model_scope` in the ruleset.
    unknown_client_policy: UnknownClientPolicy,
}

impl PrivacyEngine {
    /// Create an engine with the given rules.
    ///
    /// The unknown client policy defaults to `Allow`, preserving the
    /// out-of-the-box behaviour where all clients receive all context.
    /// Use [`from_config_file`] to load a stricter policy from disk.
    pub fn new(rules: Vec<PrivacyRule>) -> Self {
        Self {
            rules,
            unknown_client_policy: UnknownClientPolicy::Allow,
        }
    }

    /// Load rules and the unknown client policy from a TOML config file.
    ///
    /// Config format:
    /// ```toml
    /// [defaults]
    /// unknown_client = "deny"   # allow | deny  (default: allow)
    ///
    /// [[rules]]
    /// id = "block-health-from-cloud"
    /// rule_type = "sensitivity"
    /// pattern = "health"
    /// action = "block"
    /// model_scope = "claude*"
    /// ```
    ///
    /// If the file does not exist the engine starts empty (allow-all).
    pub fn from_config_file(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new(vec![]));
        }
        let contents = std::fs::read_to_string(path).map_err(|e| {
            crate::Error::Config(format!(
                "Failed to read privacy config {}: {}",
                path.display(),
                e
            ))
        })?;
        let config: PrivacyConfig = toml::from_str(&contents).map_err(|e| {
            crate::Error::Config(format!(
                "Invalid TOML in {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(Self {
            rules: config.rules,
            unknown_client_policy: config.defaults.unknown_client,
        })
    }

    pub fn rules(&self) -> &[PrivacyRule] {
        &self.rules
    }

    pub fn unknown_client_policy(&self) -> &UnknownClientPolicy {
        &self.unknown_client_policy
    }

    /// Evaluate rules for a context item against a target model.
    ///
    /// Rules are evaluated in order; the first matching rule wins.
    /// If no rule matches, the fallback depends on whether `target_model`
    /// is a *known* client (appears in at least one `model_scope`) or not:
    ///
    /// - Known client → `Allow`
    /// - Unknown client → the configured `unknown_client` policy
    ///
    /// `ContentPattern` rules are skipped here — use
    /// [`evaluate_with_content`] when item content is available.
    pub fn evaluate(
        &self,
        item_sensitivity: &str,
        item_source: &str,
        item_url: Option<&str>,
        target_model: &str,
    ) -> InjectionDecision {
        self.evaluate_with_content(item_sensitivity, item_source, item_url, None, target_model)
    }

    /// Same as [`evaluate`] but also checks `ContentPattern` rules.
    pub fn evaluate_with_content(
        &self,
        item_sensitivity: &str,
        item_source: &str,
        item_url: Option<&str>,
        item_content: Option<&str>,
        target_model: &str,
    ) -> InjectionDecision {
        for rule in &self.rules {
            // If the rule has a model scope, skip it unless the target model matches.
            if let Some(scope) = &rule.model_scope {
                if !glob_match(scope, target_model) {
                    continue;
                }
            }

            let matched = match rule.rule_type {
                RuleType::Sensitivity => glob_match(&rule.pattern, item_sensitivity),
                RuleType::Source => glob_match(&rule.pattern, item_source),
                RuleType::UrlPattern => item_url
                    .map(|u| glob_match(&rule.pattern, u))
                    .unwrap_or(false),
                RuleType::ContentPattern => item_content
                    .map(|c| c.to_lowercase().contains(&rule.pattern.to_lowercase()))
                    .unwrap_or(false),
                // BackendScope is resolved at the agent layer, not here.
                RuleType::BackendScope => false,
            };

            if matched {
                return InjectionDecision {
                    action: rule.action.clone(),
                    rules_matched: vec![rule.id.clone()],
                };
            }
        }

        // No rule matched. Fall back based on whether this client is known.
        let fallback = if self.is_known_client(target_model) {
            // Client has explicit rules configured — allow whatever wasn't blocked.
            RuleAction::Allow
        } else {
            // Client is unknown — apply the configured policy.
            match self.unknown_client_policy {
                UnknownClientPolicy::Allow => RuleAction::Allow,
                UnknownClientPolicy::Deny => RuleAction::Block,
            }
        };

        InjectionDecision {
            action: fallback,
            rules_matched: vec![],
        }
    }

    /// Returns `true` if `client_name` matches the `model_scope` of at least
    /// one rule in the ruleset. Rules without a `model_scope` do not make
    /// a client "known" — they apply to everyone regardless.
    fn is_known_client(&self, client_name: &str) -> bool {
        self.rules.iter().any(|rule| {
            rule.model_scope
                .as_ref()
                .map(|scope| glob_match(scope, client_name))
                .unwrap_or(false)
        })
    }
}

/// Case-insensitive glob matcher. Supports `*` matching zero or more
/// characters. Does not support `?` or character classes.
///
/// Examples:
/// - `"health"` matches `"health"` only
/// - `"claude*"` matches `"claude-desktop"`, `"claude-opus"`, etc.
/// - `"*"` matches everything
/// - `"*health*"` matches any value containing `"health"`
fn glob_match(pattern: &str, value: &str) -> bool {
    let p = pattern.to_lowercase();
    let v = value.to_lowercase();
    glob_inner(p.as_bytes(), v.as_bytes())
}

fn glob_inner(p: &[u8], v: &[u8]) -> bool {
    match (p.first(), v.first()) {
        // Both exhausted → full match.
        (None, None) => true,
        // Wildcard: match zero characters (advance pattern only) or one
        // character (advance value only, keep wildcard in pattern).
        (Some(&b'*'), _) => glob_inner(&p[1..], v) || (!v.is_empty() && glob_inner(p, &v[1..])),
        // Pattern exhausted but value has chars remaining → no match.
        (None, Some(_)) => false,
        // Value exhausted but pattern has non-wildcard chars → no match.
        (Some(_), None) => false,
        // Both have a char: must be equal, then recurse.
        (Some(pc), Some(vc)) => pc == vc && glob_inner(&p[1..], &v[1..]),
    }
}

/// TOML config file structure.
#[derive(serde::Deserialize, Default)]
struct PrivacyConfig {
    #[serde(default)]
    defaults: DefaultsConfig,
    #[serde(default)]
    rules: Vec<PrivacyRule>,
}

#[derive(serde::Deserialize, Default)]
struct DefaultsConfig {
    #[serde(default)]
    unknown_client: UnknownClientPolicy,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{RuleAction, RuleType, UnknownClientPolicy};

    fn rule(id: &str, rtype: RuleType, pattern: &str, action: RuleAction, scope: Option<&str>) -> PrivacyRule {
        PrivacyRule {
            id: id.to_string(),
            rule_type: rtype,
            pattern: pattern.to_string(),
            action,
            model_scope: scope.map(|s| s.to_string()),
        }
    }

    fn engine_with_policy(rules: Vec<PrivacyRule>, policy: UnknownClientPolicy) -> PrivacyEngine {
        PrivacyEngine { rules, unknown_client_policy: policy }
    }

    #[test]
    fn glob_exact() {
        assert!(glob_match("health", "health"));
        assert!(!glob_match("health", "work"));
    }

    #[test]
    fn glob_star_prefix() {
        assert!(glob_match("claude*", "claude-desktop"));
        assert!(glob_match("claude*", "claude-opus-4"));
        assert!(!glob_match("claude*", "gpt-4"));
    }

    #[test]
    fn glob_star_only() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*", ""));
    }

    #[test]
    fn glob_contains() {
        assert!(glob_match("*health*", "my-health-records"));
        assert!(!glob_match("*health*", "my-work-notes"));
    }

    #[test]
    fn glob_case_insensitive() {
        assert!(glob_match("Claude*", "claude-desktop"));
        assert!(glob_match("HEALTH", "health"));
    }

    #[test]
    fn engine_allow_all_when_no_rules() {
        let engine = PrivacyEngine::new(vec![]);
        let d = engine.evaluate("health", "shell", None, "claude-desktop");
        assert!(matches!(d.action, RuleAction::Allow));
    }

    #[test]
    fn engine_blocks_sensitivity() {
        let engine = PrivacyEngine::new(vec![
            rule("r1", RuleType::Sensitivity, "health", RuleAction::Block, None),
        ]);
        let blocked = engine.evaluate("health", "shell", None, "claude-desktop");
        assert!(matches!(blocked.action, RuleAction::Block));
        assert_eq!(blocked.rules_matched, vec!["r1"]);

        let allowed = engine.evaluate("work", "shell", None, "claude-desktop");
        assert!(matches!(allowed.action, RuleAction::Allow));
    }

    #[test]
    fn engine_model_scope_limits_rule() {
        let engine = PrivacyEngine::new(vec![
            rule("r1", RuleType::Sensitivity, "health", RuleAction::Block, Some("claude*")),
        ]);
        let d = engine.evaluate("health", "shell", None, "claude-desktop");
        assert!(matches!(d.action, RuleAction::Block));

        // ollama has no matching model_scope → it's unknown → Allow (default policy)
        let d2 = engine.evaluate("health", "shell", None, "ollama-llama3");
        assert!(matches!(d2.action, RuleAction::Allow));
    }

    #[test]
    fn engine_content_pattern_requires_content() {
        let engine = PrivacyEngine::new(vec![
            rule("r1", RuleType::ContentPattern, "salary", RuleAction::Block, None),
        ]);
        let d = engine.evaluate("work", "notes", None, "claude");
        assert!(matches!(d.action, RuleAction::Allow));

        let d2 = engine.evaluate_with_content(
            "work", "notes", None,
            Some("My salary is $200k"),
            "claude",
        );
        assert!(matches!(d2.action, RuleAction::Block));
    }

    #[test]
    fn engine_first_rule_wins() {
        let engine = PrivacyEngine::new(vec![
            rule("r1", RuleType::Sensitivity, "health", RuleAction::Summarize, None),
            rule("r2", RuleType::Sensitivity, "health", RuleAction::Block, None),
        ]);
        let d = engine.evaluate("health", "shell", None, "any");
        assert!(matches!(d.action, RuleAction::Summarize));
        assert_eq!(d.rules_matched, vec!["r1"]);
    }

    // --- Unknown client policy tests ---

    #[test]
    fn unknown_client_allow_policy_is_default() {
        // No rules at all → client is unknown → default Allow policy
        let engine = PrivacyEngine::new(vec![]);
        let d = engine.evaluate("health", "shell", None, "mystery-client");
        assert!(matches!(d.action, RuleAction::Allow));
    }

    #[test]
    fn unknown_client_deny_policy_blocks_unrecognised() {
        let engine = engine_with_policy(
            vec![rule("r1", RuleType::Sensitivity, "health", RuleAction::Block, Some("claude*"))],
            UnknownClientPolicy::Deny,
        );
        // "mystery-client" has no model_scope match → unknown → Deny policy → Block
        let d = engine.evaluate("work", "shell", None, "mystery-client");
        assert!(matches!(d.action, RuleAction::Block));
    }

    #[test]
    fn known_client_with_deny_policy_still_gets_allow_fallback() {
        // claude* is a known client (has rules scoped to it).
        // For items that no rule matches, known clients get Allow — not Deny.
        let engine = engine_with_policy(
            vec![rule("r1", RuleType::Sensitivity, "health", RuleAction::Block, Some("claude*"))],
            UnknownClientPolicy::Deny,
        );
        // "work" item for claude: no rule matches (only health is blocked) → known client → Allow
        let d = engine.evaluate("work", "shell", None, "claude-desktop");
        assert!(matches!(d.action, RuleAction::Allow));
    }

    #[test]
    fn is_known_client_requires_model_scope() {
        // A rule with no model_scope does NOT make any client "known"
        let engine = PrivacyEngine::new(vec![
            rule("r1", RuleType::Sensitivity, "health", RuleAction::Block, None),
        ]);
        assert!(!engine.is_known_client("claude-desktop"));
        assert!(!engine.is_known_client("anything"));
    }

    #[test]
    fn is_known_client_matches_scope_glob() {
        let engine = PrivacyEngine::new(vec![
            rule("r1", RuleType::Sensitivity, "health", RuleAction::Block, Some("claude*")),
        ]);
        assert!(engine.is_known_client("claude-desktop"));
        assert!(engine.is_known_client("claude-opus-4"));
        assert!(!engine.is_known_client("gpt-4"));
    }
}
