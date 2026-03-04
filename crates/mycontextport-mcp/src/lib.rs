//! MCP stdio server for MyContextPort.
//!
//! Implements the Model Context Protocol over stdio transport, which is
//! what Claude Desktop uses when launching an MCP server as a subprocess.
//! The server reads newline-delimited JSON-RPC from stdin and writes
//! responses to stdout.

use anyhow::Result;
use mycontextport_privacy::{rule::RuleAction, PrivacyEngine};
use mycontextport_store::ContextStore;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info, warn};

/// Start the MCP stdio server. Blocks until stdin is closed (client disconnects).
///
/// The `engine` evaluates privacy rules before each context injection.
/// Pass `Arc::new(PrivacyEngine::new(vec![]))` for allow-all (no rules).
pub async fn serve_stdio(store: Arc<ContextStore>, engine: Arc<PrivacyEngine>) -> Result<()> {
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

        if let Some(response) = handle_request(&request, &store, &engine, &client_name) {
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
                "tools": [{
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
                }]
            }
        })),

        "tools/call" => {
            let params = request.get("params")?;
            let tool_name = params.get("name")?.as_str()?;

            match tool_name {
                "get_context" => {
                    let limit = params
                        .get("arguments")
                        .and_then(|a| a.get("limit"))
                        .and_then(|l| l.as_u64())
                        .unwrap_or(20)
                        .min(200) as usize;

                    // Fetch more than requested so filtering doesn't leave us short.
                    let fetch_limit = (limit * 3).max(limit + 50).min(500);

                    match store.query_recent(fetch_limit) {
                        Ok(all_items) => {
                            let mut allowed: Vec<&mycontextport_store::ContextItem> = Vec::new();
                            let mut injected_ids: Vec<String> = Vec::new();
                            let mut blocked_ids: Vec<String> = Vec::new();
                            let mut rules_fired: Vec<String> = Vec::new();

                            for item in &all_items {
                                if allowed.len() >= limit {
                                    break;
                                }

                                let decision = engine.evaluate_with_content(
                                    item.sensitivity.as_str(),
                                    &item.source,
                                    item.url.as_deref(),
                                    Some(item.content.as_str()),
                                    client_name,
                                );

                                rules_fired.extend(decision.rules_matched);

                                match decision.action {
                                    RuleAction::Block => {
                                        blocked_ids.push(item.id.clone());
                                    }
                                    RuleAction::Allow | RuleAction::Summarize => {
                                        injected_ids.push(item.id.clone());
                                        allowed.push(item);
                                    }
                                }
                            }

                            // Write to audit log (fire-and-forget — don't fail the
                            // request if logging fails).
                            let used_refs: Vec<&str> =
                                injected_ids.iter().map(|s| s.as_str()).collect();
                            let blocked_refs: Vec<&str> =
                                blocked_ids.iter().map(|s| s.as_str()).collect();
                            let rules_refs: Vec<&str> =
                                rules_fired.iter().map(|s| s.as_str()).collect();
                            if let Err(e) =
                                store.log_injection(client_name, &used_refs, &blocked_refs, &rules_refs)
                            {
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
