//! MCP stdio server for ContextGenOS.
//!
//! Implements the Model Context Protocol over stdio transport, which is
//! what Claude Desktop uses when launching an MCP server as a subprocess.
//! The server reads newline-delimited JSON-RPC from stdin and writes
//! responses to stdout.

use anyhow::Result;
use contextgenos_store::ContextStore;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info, warn};

/// Start the MCP stdio server. Blocks until stdin is closed (client disconnects).
pub async fn serve_stdio(store: Arc<ContextStore>) -> Result<()> {
    info!("MCP stdio server starting");

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

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

        if let Some(response) = handle_request(&request, &store) {
            let mut response_str = serde_json::to_string(&response)?;
            response_str.push('\n');
            stdout.write_all(response_str.as_bytes()).await?;
            stdout.flush().await?;
            debug!(message = %response_str.trim(), "MCP → sent");
        }
    }

    Ok(())
}

fn handle_request(request: &Value, store: &Arc<ContextStore>) -> Option<Value> {
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
                    "name": "contextgenos",
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
                    "description": "Retrieve recent personal context items collected by ContextGenOS (shell history and other local activity).",
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

                    match store.query_recent(limit) {
                        Ok(items) => {
                            let text = if items.is_empty() {
                                "No context items found. ContextGenOS is running but no items have been collected yet.".to_string()
                            } else {
                                let mut out = format!("Recent activity ({} items):\n\n", items.len());
                                for item in &items {
                                    let ts = chrono::DateTime::from_timestamp(item.collected_at, 0)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                        .unwrap_or_else(|| item.collected_at.to_string());
                                    out.push_str(&format!("[{}] {} — {}\n", item.source, ts, item.content));
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
