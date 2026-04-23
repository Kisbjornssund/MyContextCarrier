//! MCP stdio server for MyContextPort.
//!
//! Implements the Model Context Protocol over stdio transport, which is
//! what Claude Desktop uses when launching an MCP server as a subprocess.
//! The server reads newline-delimited JSON-RPC from stdin and writes
//! responses to stdout.

use anyhow::Result;
use mycontextport_graph::GraphIndexer;
use mycontextport_privacy::{
    rule::RuleAction, GuardrailDecision, GuardrailTrigger, GuardrailsEngine, PrivacyEngine,
};
use mycontextport_store::{ContextStore, TraceStep};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info, warn};

/// Minimal glob match: supports `*` as wildcard. Case-insensitive.
fn glob_match(pattern: &str, value: &str) -> bool {
    let p = pattern.to_lowercase();
    let v = value.to_lowercase();
    if p == "*" { return true; }
    if !p.contains('*') { return p == v; }
    let parts: Vec<&str> = p.split('*').collect();
    let mut pos = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() { continue; }
        if i == 0 {
            if !v.starts_with(part.as_ref() as &str) { return false; }
            pos = part.len();
        } else if let Some(idx) = v[pos..].find(part.as_ref() as &str) {
            pos += idx + part.len();
        } else {
            return false;
        }
    }
    if p.ends_with('*') { true } else { pos == v.len() || v[pos..].is_empty() }
}

/// Start the MCP stdio server. Blocks until stdin is closed (client disconnects).
pub async fn serve_stdio(store: Arc<ContextStore>, engine: Arc<PrivacyEngine>) -> Result<()> {
    serve_stdio_with_guardrails(store, engine, Arc::new(GuardrailsEngine::new(vec![]))).await
}

pub async fn serve_stdio_with_guardrails(
    store: Arc<ContextStore>,
    engine: Arc<PrivacyEngine>,
    guardrails: Arc<GuardrailsEngine>,
) -> Result<()> {
    info!("MCP stdio server starting");

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    // Tracks the name of the connected MCP client (set on `initialize`).
    // Used as the `target_model` passed to the privacy engine.
    let mut client_name = String::from("unknown");

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            info!("MCP stdin closed, shutting down");
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        debug!(message = %trimmed, "MCP ← received");

        let request: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                error!(error = %e, raw = %trimmed, "Failed to parse JSON-RPC message");
                continue;
            }
        };

        // Capture client name from the initialize handshake so the privacy
        // engine can apply per-model rules.
        if request.get("method").and_then(|m| m.as_str()) == Some("initialize") {
            if let Some(name) = request
                .get("params")
                .and_then(|p| p.get("clientInfo"))
                .and_then(|ci| ci.get("name"))
                .and_then(|n| n.as_str())
            {
                client_name = name.to_string();
                info!(client = %client_name, "MCP client connected");
            }
        }

        if let Some(response) = handle_request(&request, &store, &engine, &guardrails, &client_name) {
            let mut response_str = serde_json::to_string(&response)?;
            response_str.push('\n');
            stdout.write_all(response_str.as_bytes()).await?;
            stdout.flush().await?;
            debug!(message = %response_str.trim(), "MCP → sent");
        }
    }

    Ok(())
}

fn handle_request(
    request: &Value,
    store: &Arc<ContextStore>,
    engine: &PrivacyEngine,
    guardrails: &GuardrailsEngine,
    client_name: &str,
) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method")?.as_str()?;

    match method {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "mycontextport",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })),

        // Notifications have no id and must not receive a response.
        "notifications/initialized" => {
            info!("MCP client initialized");
            None
        }

        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": [
                    {
                        "name": "get_context",
                        "description": "Retrieve recent personal context items collected by MyContextPort (shell history and other local activity). Items are filtered by privacy rules before being returned.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "limit": {
                                    "type": "integer",
                                    "description": "Maximum number of items to return (default: 20, max: 200)",
                                    "default": 20
                                }
                            },
                            "required": []
                        }
                    },
                    {
                        "name": "append_context",
                        "description": "Store a new context item into MyContextPort. Use this to remember decisions, notes, or any information that should persist across sessions.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "content": {
                                    "type": "string",
                                    "description": "The text to store"
                                },
                                "source": {
                                    "type": "string",
                                    "description": "Label for where this came from (e.g. 'agent', 'user', 'meeting-notes')"
                                },
                                "sensitivity": {
                                    "type": "string",
                                    "description": "Sensitivity level: unknown, public, work, personal, health, financial",
                                    "default": "unknown"
                                },
                                "url": {
                                    "type": "string",
                                    "description": "Optional URL or file path associated with this item"
                                }
                            },
                            "required": ["content", "source"]
                        }
                    },
                    {
                        "name": "list_sources",
                        "description": "List all context sources and how many items each has collected. Useful before calling get_context to understand what data is available.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    },
                    {
                        "name": "list_threads",
                        "description": "List all memory threads accessible to this client. Threads are project-scoped context groups with custom instructions.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    },
                    {
                        "name": "get_thread_context",
                        "description": "Retrieve context items belonging to a specific memory thread, along with the thread's custom instructions.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "thread_id": { "type": "string", "description": "Thread ID from list_threads" },
                                "limit": { "type": "integer", "description": "Max items to return (default 20)", "default": 20 }
                            },
                            "required": ["thread_id"]
                        }
                    },
                    {
                        "name": "create_thread",
                        "description": "Create a new memory thread for organising context around a project or topic.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string", "description": "Thread name" },
                                "description": { "type": "string" },
                                "instructions": { "type": "string", "description": "Custom instructions injected with this thread's context" }
                            },
                            "required": ["name"]
                        }
                    },
                    {
                        "name": "assign_to_thread",
                        "description": "Assign an existing context item to a memory thread.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "item_id": { "type": "string" },
                                "thread_id": { "type": "string" }
                            },
                            "required": ["item_id", "thread_id"]
                        }
                    },
                    {
                        "name": "search_context",
                        "description": "Search for context items related to a query using the entity knowledge graph. Falls back to recent items if the graph has not been built yet.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "Entity name, project, tool, or keyword to search for"
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "Maximum number of items to return (default: 20)",
                                    "default": 20
                                }
                            },
                            "required": ["query"]
                        }
                    }
                ]
            }
        })),

        "tools/call" => {
            let params = request.get("params")?;
            let tool_name = params.get("name")?.as_str()?;

            match tool_name {
                "get_context" => {
                    let t_start = Instant::now();
                    let limit = params
                        .get("arguments")
                        .and_then(|a| a.get("limit"))
                        .and_then(|l| l.as_u64())
                        .unwrap_or(20)
                        .min(200) as usize;

                    let fetch_limit = (limit * 3).max(limit + 50).min(500);

                    match store.query_recent(fetch_limit) {
                        Ok(all_items) => {
                            let t_fetch = t_start.elapsed().as_millis() as u64;
                            let mut allowed: Vec<&mycontextport_store::ContextItem> = Vec::new();
                            let mut injected_ids: Vec<String> = Vec::new();
                            let mut blocked_ids: Vec<String> = Vec::new();
                            let mut rules_fired: Vec<String> = Vec::new();

                            let t_eval_start = Instant::now();
                            for item in &all_items {
                                if allowed.len() >= limit { break; }
                                let decision = engine.evaluate_with_content(
                                    item.sensitivity.as_str(), &item.source,
                                    item.url.as_deref(), Some(item.content.as_str()), client_name,
                                );
                                rules_fired.extend(decision.rules_matched);
                                match decision.action {
                                    RuleAction::Block => { blocked_ids.push(item.id.clone()); }
                                    RuleAction::Allow | RuleAction::Summarize => {
                                        injected_ids.push(item.id.clone());
                                        allowed.push(item);
                                    }
                                }
                            }
                            let t_eval = t_eval_start.elapsed().as_millis() as u64;
                            let t_total = t_start.elapsed().as_millis() as u64;

                            let trace = vec![
                                TraceStep { step: "fetch".into(), duration_ms: t_fetch, detail: json!({"fetched": all_items.len()}) },
                                TraceStep { step: "privacy_eval".into(), duration_ms: t_eval, detail: json!({"allowed": injected_ids.len(), "blocked": blocked_ids.len()}) },
                                TraceStep { step: "inject".into(), duration_ms: t_total, detail: json!({"items": injected_ids.len()}) },
                            ];

                            let used_refs: Vec<&str> = injected_ids.iter().map(|s| s.as_str()).collect();
                            let blocked_refs: Vec<&str> = blocked_ids.iter().map(|s| s.as_str()).collect();
                            let rules_refs: Vec<&str> = rules_fired.iter().map(|s| s.as_str()).collect();
                            if let Err(e) = store.log_injection_full(client_name, &used_refs, &blocked_refs, &rules_refs, None, &trace) {
                                warn!(error = %e, "Failed to write injection log");
                            }

                            let text = if allowed.is_empty() {
                                if blocked_ids.is_empty() {
                                    "No context items found. MyContextPort is running but no items have been collected yet.".to_string()
                                } else {
                                    format!(
                                        "All {} available context item(s) were blocked by privacy rules.",
                                        blocked_ids.len()
                                    )
                                }
                            } else {
                                let mut out = format!(
                                    "Recent activity ({} item(s)",
                                    allowed.len()
                                );
                                if !blocked_ids.is_empty() {
                                    out.push_str(&format!(
                                        ", {} blocked by privacy rules",
                                        blocked_ids.len()
                                    ));
                                }
                                out.push_str("):\n\n");
                                for item in &allowed {
                                    let ts = chrono::DateTime::from_timestamp(item.collected_at, 0)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                        .unwrap_or_else(|| item.collected_at.to_string());
                                    out.push_str(&format!(
                                        "[{}] {} — {}\n",
                                        item.source, ts, item.content
                                    ));
                                }
                                out
                            };

                            Some(json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "content": [{"type": "text", "text": text}]
                                }
                            }))
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to query store");
                            Some(json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {
                                    "code": -32603,
                                    "message": format!("Store error: {}", e)
                                }
                            }))
                        }
                    }
                }

                "append_context" => {
                    let args = params.get("arguments").cloned().unwrap_or(json!({}));
                    let content = match args.get("content").and_then(|v| v.as_str()) {
                        Some(c) if !c.trim().is_empty() => c.to_string(),
                        _ => {
                            return Some(json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {"code": -32602, "message": "content is required"}
                            }));
                        }
                    };
                    let source = args
                        .get("source")
                        .and_then(|v| v.as_str())
                        .unwrap_or("agent")
                        .to_string();
                    let sensitivity_str = args
                        .get("sensitivity")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let sensitivity = sensitivity_str
                        .parse::<mycontextport_store::Sensitivity>()
                        .unwrap_or_default();
                    let url = args.get("url").and_then(|v| v.as_str()).map(|s| s.to_string());

                    // Guardrail check before storing
                    let content = match guardrails.evaluate(GuardrailTrigger::WriteBack, &content) {
                        GuardrailDecision::Allow => content,
                        GuardrailDecision::Redact { cleaned_content } => cleaned_content,
                        GuardrailDecision::Block { message } => {
                            return Some(json!({
                                "jsonrpc": "2.0", "id": id,
                                "error": {"code": -32603, "message": format!("Blocked by guardrail: {}", message)}
                            }));
                        }
                        GuardrailDecision::RequireConfirmation { message } => {
                            return Some(json!({
                                "jsonrpc": "2.0", "id": id,
                                "error": {
                                    "code": -32000,
                                    "message": message,
                                    "data": {"guardrail_triggered": true, "retry_with_confirmation": true}
                                }
                            }));
                        }
                    };

                    let item = mycontextport_store::ContextItem {
                        id: uuid::Uuid::new_v4().to_string(),
                        content: content.clone(),
                        source: source.clone(),
                        collected_at: chrono::Utc::now().timestamp(),
                        url,
                        sensitivity,
                        metadata: serde_json::Value::Object(Default::default()),
                    };

                    match store.insert_items(&[item]) {
                        Ok(_) => {
                            info!(source = %source, "append_context: stored item");
                            Some(json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "content": [{"type": "text", "text": "Context stored."}]
                                }
                            }))
                        }
                        Err(e) => {
                            error!(error = %e, "append_context: store error");
                            Some(json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {"code": -32603, "message": format!("Store error: {}", e)}
                            }))
                        }
                    }
                }

                "list_threads" => {
                    match store.list_threads(false) {
                        Ok(threads) => {
                            let accessible: Vec<_> = threads
                                .into_iter()
                                .filter(|t| {
                                    t.agent_scope.as_deref()
                                        .map(|scope| glob_match(scope, client_name))
                                        .unwrap_or(true)
                                })
                                .collect();
                            let text = if accessible.is_empty() {
                                "No threads found.".to_string()
                            } else {
                                accessible.iter().map(|t| {
                                    let desc = t.description.as_deref().unwrap_or("");
                                    format!("[{}] {} — {}", t.id, t.name, desc)
                                }).collect::<Vec<_>>().join("\n")
                            };
                            Some(json!({
                                "jsonrpc": "2.0", "id": id,
                                "result": { "content": [{"type": "text", "text": text}] }
                            }))
                        }
                        Err(e) => Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": {"code": -32603, "message": format!("Store error: {}", e)}
                        }))
                    }
                }

                "get_thread_context" => {
                    let args = params.get("arguments").cloned().unwrap_or(json!({}));
                    let thread_id = match args.get("thread_id").and_then(|v| v.as_str()) {
                        Some(id) => id.to_string(),
                        None => return Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": {"code": -32602, "message": "thread_id is required"}
                        })),
                    };
                    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20).min(200) as usize;

                    let thread = match store.get_thread(&thread_id) {
                        Ok(Some(t)) => t,
                        Ok(None) => return Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": {"code": -32602, "message": "Thread not found"}
                        })),
                        Err(e) => return Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": {"code": -32603, "message": format!("Store error: {}", e)}
                        })),
                    };

                    // Check agent_scope
                    if let Some(scope) = &thread.agent_scope {
                        if !glob_match(scope, client_name) {
                            return Some(json!({
                                "jsonrpc": "2.0", "id": id,
                                "error": {"code": -32603, "message": "Thread not accessible to this client"}
                            }));
                        }
                    }

                    let items = store.query_thread_items(&thread_id, limit * 3).unwrap_or_default();
                    let mut allowed = vec![];
                    let mut blocked_ids = vec![];
                    let mut rules_fired = vec![];

                    for item in &items {
                        if allowed.len() >= limit { break; }
                        let decision = engine.evaluate_with_content(
                            item.sensitivity.as_str(), &item.source,
                            item.url.as_deref(), Some(&item.content), client_name,
                        );
                        rules_fired.extend(decision.rules_matched);
                        match decision.action {
                            RuleAction::Block => { blocked_ids.push(item.id.clone()); }
                            _ => { allowed.push(item); }
                        }
                    }

                    let used: Vec<&str> = allowed.iter().map(|i| i.id.as_str()).collect();
                    let blocked: Vec<&str> = blocked_ids.iter().map(|s| s.as_str()).collect();
                    let rules: Vec<&str> = rules_fired.iter().map(|s| s.as_str()).collect();
                    let _ = store.log_injection(client_name, &used, &blocked, &rules);

                    let mut text = String::new();
                    if let Some(instr) = &thread.instructions {
                        text.push_str(&format!("Thread instructions: {}\n\n", instr));
                    }
                    if allowed.is_empty() {
                        text.push_str("No context items in this thread.");
                    } else {
                        for item in &allowed {
                            let ts = chrono::DateTime::from_timestamp(item.collected_at, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                .unwrap_or_default();
                            text.push_str(&format!("[{}] {} — {}\n", item.source, ts, item.content));
                        }
                    }

                    Some(json!({
                        "jsonrpc": "2.0", "id": id,
                        "result": { "content": [{"type": "text", "text": text}] }
                    }))
                }

                "create_thread" => {
                    let args = params.get("arguments").cloned().unwrap_or(json!({}));
                    let name = match args.get("name").and_then(|v| v.as_str()) {
                        Some(n) if !n.trim().is_empty() => n.to_string(),
                        _ => return Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": {"code": -32602, "message": "name is required"}
                        })),
                    };
                    let description = args.get("description").and_then(|v| v.as_str());
                    let instructions = args.get("instructions").and_then(|v| v.as_str());

                    // Guardrail check on thread name + instructions
                    let check_text = format!("{} {}", name, instructions.unwrap_or(""));
                    match guardrails.evaluate(GuardrailTrigger::ThreadCreate, &check_text) {
                        GuardrailDecision::Block { message } => {
                            return Some(json!({
                                "jsonrpc": "2.0", "id": id,
                                "error": {"code": -32603, "message": format!("Blocked by guardrail: {}", message)}
                            }));
                        }
                        GuardrailDecision::RequireConfirmation { message } => {
                            return Some(json!({
                                "jsonrpc": "2.0", "id": id,
                                "error": {
                                    "code": -32000,
                                    "message": message,
                                    "data": {"guardrail_triggered": true}
                                }
                            }));
                        }
                        _ => {}
                    }

                    match store.create_thread(&name, description, None, instructions) {
                        Ok(thread) => {
                            info!(thread_id = %thread.id, name = %name, "Thread created");
                            Some(json!({
                                "jsonrpc": "2.0", "id": id,
                                "result": {
                                    "content": [{"type": "text", "text": format!("Thread created: {} (id: {})", name, thread.id)}]
                                }
                            }))
                        }
                        Err(e) => Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": {"code": -32603, "message": format!("Store error: {}", e)}
                        }))
                    }
                }

                "assign_to_thread" => {
                    let args = params.get("arguments").cloned().unwrap_or(json!({}));
                    let item_id = args.get("item_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let thread_id = args.get("thread_id").and_then(|v| v.as_str()).unwrap_or("").to_string();

                    if item_id.is_empty() || thread_id.is_empty() {
                        return Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": {"code": -32602, "message": "item_id and thread_id are required"}
                        }));
                    }

                    match store.assign_item_to_thread(&item_id, &thread_id) {
                        Ok(()) => Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "result": { "content": [{"type": "text", "text": "Item assigned to thread."}] }
                        })),
                        Err(e) => Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": {"code": -32603, "message": format!("Store error: {}", e)}
                        }))
                    }
                }

                "list_sources" => {
                    match store.items_by_source() {
                        Ok(map) => {
                            let mut sources: Vec<_> = map.into_iter().collect();
                            sources.sort_by(|a, b| b.1.cmp(&a.1));
                            let text = if sources.is_empty() {
                                "No context collected yet.".to_string()
                            } else {
                                sources
                                    .iter()
                                    .map(|(src, count)| format!("{}: {} item(s)", src, count))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            };
                            Some(json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "content": [{"type": "text", "text": text}]
                                }
                            }))
                        }
                        Err(e) => {
                            error!(error = %e, "list_sources: store error");
                            Some(json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "error": {"code": -32603, "message": format!("Store error: {}", e)}
                            }))
                        }
                    }
                }

                "search_context" => {
                    let args = params.get("arguments").cloned().unwrap_or(json!({}));
                    let query = match args.get("query").and_then(|v| v.as_str()) {
                        Some(q) if !q.trim().is_empty() => q.to_string(),
                        _ => return Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": {"code": -32602, "message": "query is required"}
                        })),
                    };
                    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20).min(200) as usize;

                    let indexer = GraphIndexer::new(store.clone());
                    let fetch_limit = (limit * 3).max(limit + 50).min(500);

                    match indexer.search(&query, fetch_limit) {
                        Ok(all_items) => {
                            let mut allowed = vec![];
                            let mut blocked_ids = vec![];
                            let mut rules_fired = vec![];

                            for item in &all_items {
                                if allowed.len() >= limit { break; }
                                let decision = engine.evaluate_with_content(
                                    item.sensitivity.as_str(), &item.source,
                                    item.url.as_deref(), Some(&item.content), client_name,
                                );
                                rules_fired.extend(decision.rules_matched);
                                match decision.action {
                                    RuleAction::Block => { blocked_ids.push(item.id.clone()); }
                                    _ => { allowed.push(item); }
                                }
                            }

                            let used: Vec<&str> = allowed.iter().map(|i| i.id.as_str()).collect();
                            let blocked: Vec<&str> = blocked_ids.iter().map(|s| s.as_str()).collect();
                            let rules: Vec<&str> = rules_fired.iter().map(|s| s.as_str()).collect();
                            let _ = store.log_injection(client_name, &used, &blocked, &rules);

                            let text = if allowed.is_empty() {
                                format!("No context items found related to '{}'.", query)
                            } else {
                                let mut out = format!("Context related to '{}' ({} item(s)):\n\n", query, allowed.len());
                                for item in &allowed {
                                    let ts = chrono::DateTime::from_timestamp(item.collected_at, 0)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                        .unwrap_or_default();
                                    out.push_str(&format!("[{}] {} — {}\n", item.source, ts, item.content));
                                }
                                out
                            };

                            Some(json!({
                                "jsonrpc": "2.0", "id": id,
                                "result": { "content": [{"type": "text", "text": text}] }
                            }))
                        }
                        Err(e) => Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": {"code": -32603, "message": format!("Search error: {}", e)}
                        }))
                    }
                }

                unknown => {
                    warn!(tool = unknown, "Unknown tool called");
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": format!("Unknown tool: {}", unknown)
                        }
                    }))
                }
            }
        }

        other => {
            // Unknown method: return error if it has an id (request),
            // silently ignore if it has no id (notification).
            if id != Value::Null {
                warn!(method = other, "Unknown JSON-RPC method");
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", other)
                    }
                }))
            } else {
                None
            }
        }
    }
}
