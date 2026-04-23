//! Privacy rules engine.
//!
//! Enforces user-defined rules determining what context can be injected
//! into which AI models. Rules are evaluated before any context leaves
//! the local store.

pub mod engine;
pub mod error;
pub mod guardrails;
pub mod rule;

pub use engine::PrivacyEngine;
pub use error::Error;
pub use guardrails::{Guardrail, GuardrailAction, GuardrailDecision, GuardrailTrigger, GuardrailsEngine};
pub use rule::{PrivacyRule, RuleAction, UnknownClientPolicy};

pub type Result<T> = std::result::Result<T, Error>;
