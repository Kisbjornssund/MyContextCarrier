use anyhow::Result;
use mycontextport_store::ContextStore;
use std::sync::Arc;
use tracing::{debug, warn};

use crate::extractor::extract_entities;

pub struct GraphIndexer {
    store: Arc<ContextStore>,
}

impl GraphIndexer {
    pub fn new(store: Arc<ContextStore>) -> Self {
        Self { store }
    }

    /// Extract entities from one context item and link them in the graph.
    /// The edge's `item_id` column records which item produced it.
    pub fn index_item(
        &self,
        item_id: &str,
        source: &str,
        content: &str,
        metadata: &serde_json::Value,
    ) -> Result<usize> {
        let entities = extract_entities(source, content, metadata);
        let mut linked = 0usize;

        for entity in &entities {
            let node_id = match self.store.upsert_graph_node(
                entity.kind.as_str(),
                &entity.name,
                &serde_json::json!({"source": source}),
            ) {
                Ok(id) => id,
                Err(e) => {
                    warn!(error = %e, name = %entity.name, "Failed to upsert graph node");
                    continue;
                }
            };

            if let Err(e) = self.store.upsert_graph_edge(item_id, &node_id, "mentions", Some(item_id)) {
                warn!(error = %e, "Failed to upsert graph edge");
            } else {
                linked += 1;
            }
        }

        debug!(item = %item_id, entities = linked, "Indexed item");
        Ok(linked)
    }

    /// Index all items in the store. Upserts are idempotent, so re-running is safe.
    pub fn index_all(&self) -> Result<usize> {
        let items = self.store.query_recent(5000)?;
        let mut total = 0usize;
        for item in &items {
            total += self.index_item(&item.id, &item.source, &item.content, &item.metadata)?;
        }
        Ok(total)
    }

    /// Search for context items related to `query` via the entity graph.
    /// Falls back to `query_recent` if the graph is empty.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<mycontextport_store::ContextItem>> {
        let node_count = self.store.graph_node_count().unwrap_or(0);

        if node_count == 0 {
            return self.store.query_recent(limit);
        }

        let nodes = self.store.search_graph_nodes(query)?;

        if nodes.is_empty() {
            return self.store.query_recent(limit);
        }

        let per_node = ((limit / nodes.len()) + 1).max(3);
        let mut seen = std::collections::HashSet::<String>::new();
        let mut results: Vec<mycontextport_store::ContextItem> = Vec::new();

        for node in &nodes {
            let items = self.store.items_for_node(&node.id, per_node)?;
            for item in items {
                if seen.insert(item.id.clone()) {
                    results.push(item);
                    if results.len() >= limit {
                        return Ok(results);
                    }
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycontextport_store::{ContextItem, ContextStore, Sensitivity};
    use serde_json::json;

    fn in_memory_store() -> Arc<ContextStore> {
        Arc::new(ContextStore::open(":memory:").expect("in-memory store"))
    }

    #[test]
    fn indexes_item_and_finds_it() {
        let store = in_memory_store();
        let indexer = GraphIndexer::new(store.clone());

        let item = ContextItem {
            id: uuid::Uuid::new_v4().to_string(),
            content: "cargo build --release".to_string(),
            source: "shell-history".to_string(),
            collected_at: 0,
            url: None,
            sensitivity: Sensitivity::Public,
            metadata: json!({}),
        };
        store.insert_items(&[item.clone()]).unwrap();

        let linked = indexer.index_item(&item.id, &item.source, &item.content, &item.metadata).unwrap();
        assert!(linked > 0, "should have linked at least one entity (cargo)");

        let results = indexer.search("cargo", 10).unwrap();
        assert!(!results.is_empty(), "search should find the indexed item");
    }

    #[test]
    fn falls_back_when_graph_empty() {
        let store = in_memory_store();
        let indexer = GraphIndexer::new(store.clone());

        let item = ContextItem {
            id: uuid::Uuid::new_v4().to_string(),
            content: "some note".to_string(),
            source: "notes".to_string(),
            collected_at: 0,
            url: None,
            sensitivity: Sensitivity::Public,
            metadata: json!({}),
        };
        store.insert_items(&[item]).unwrap();

        let results = indexer.search("anything", 10).unwrap();
        assert!(!results.is_empty(), "should fall back to query_recent");
    }
}
