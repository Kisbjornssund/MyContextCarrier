//! Knowledge graph over collected context items.
//!
//! Extracts entities (people, projects, tools, concepts) from context items
//! using rule-based patterns — no LLM dependency, fully local.
//!
//! After extraction, entities are stored as graph nodes in the context store
//! and linked to the items they appeared in via graph edges.

pub mod extractor;
pub mod indexer;

pub use extractor::{Entity, EntityKind, extract_entities};
pub use indexer::GraphIndexer;
