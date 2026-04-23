//! Localhost web dashboard for MyContextPort.
//!
//! Exposes a small axum HTTP server on 127.0.0.1:<port> that serves:
//!   GET /              — embedded HTML dashboard (no external deps)
//!   GET /api/status    — { items_total, items_by_source, store_path }
//!   GET /api/items     — recent ContextItems (query: limit, source)
//!   GET /api/log       — recent InjectionLogEntries (query: limit)
//!   GET /api/privacy   — loaded privacy rules + unknown_client_policy

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use mycontextport_privacy::PrivacyEngine;
use mycontextport_store::{CollectionRun, ContextStore, InjectionLogEntry};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::cors::CorsLayer;

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    store: Arc<ContextStore>,
    engine: Arc<PrivacyEngine>,
    store_path: PathBuf,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct StatusResponse {
    items_total: i64,
    items_by_source: HashMap<String, i64>,
    store_path: String,
}

#[derive(Deserialize)]
struct ItemsQuery {
    #[serde(default = "default_limit")]
    limit: usize,
    source: Option<String>,
}

#[derive(Deserialize)]
struct LogQuery {
    #[serde(default = "default_log_limit")]
    limit: usize,
}

#[derive(Deserialize)]
struct RunsQuery {
    #[serde(default = "default_log_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    50
}
fn default_log_limit() -> usize {
    20
}

#[derive(Serialize)]
struct PrivacyResponse {
    rules: Vec<serde_json::Value>,
    unknown_client_policy: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn handler_index() -> Response {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        DASHBOARD_HTML,
    )
        .into_response()
}

async fn handler_status(State(state): State<AppState>) -> Result<Json<StatusResponse>, StatusCode> {
    let total = state.store.count().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let by_source = state
        .store
        .items_by_source()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(StatusResponse {
        items_total: total,
        items_by_source: by_source,
        store_path: state.store_path.display().to_string(),
    }))
}

async fn handler_items(
    State(state): State<AppState>,
    Query(params): Query<ItemsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let limit = params.limit.min(500);
    let items = state
        .store
        .query_recent(limit)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let items: Vec<_> = if let Some(src) = &params.source {
        items.into_iter().filter(|i| &i.source == src).collect()
    } else {
        items
    };
    Ok(Json(items))
}

async fn handler_log(
    State(state): State<AppState>,
    Query(params): Query<LogQuery>,
) -> Result<Json<Vec<InjectionLogEntry>>, StatusCode> {
    let limit = params.limit.min(200);
    let entries = state
        .store
        .query_log(limit)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(entries))
}

async fn handler_log_trace(
    State(state): State<AppState>,
    Path(log_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let entries = state.store.query_log(500).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let entry = entries.into_iter().find(|e| e.id == log_id);
    match entry {
        Some(e) => Ok(Json(e.trace)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn handler_runs(
    State(state): State<AppState>,
    Query(params): Query<RunsQuery>,
) -> Result<Json<Vec<CollectionRun>>, StatusCode> {
    let runs = state
        .store
        .query_runs(params.limit.min(200))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(runs))
}

async fn handler_privacy(State(state): State<AppState>) -> Json<PrivacyResponse> {
    let rules: Vec<serde_json::Value> = state
        .engine
        .rules()
        .iter()
        .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
        .collect();
    let policy = format!("{:?}", state.engine.unknown_client_policy()).to_lowercase();
    Json(PrivacyResponse {
        rules,
        unknown_client_policy: policy,
    })
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn serve(
    store: Arc<ContextStore>,
    engine: Arc<PrivacyEngine>,
    store_path: PathBuf,
    port: u16,
    no_open: bool,
) -> anyhow::Result<()> {
    let state = AppState { store, engine, store_path };
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let url = format!("http://localhost:{port}");

    let app = Router::new()
        .route("/", get(handler_index))
        .route("/api/status", get(handler_status))
        .route("/api/items", get(handler_items))
        .route("/api/log", get(handler_log))
        .route("/api/log/:id/trace", get(handler_log_trace))
        .route("/api/runs", get(handler_runs))
        .route("/api/privacy", get(handler_privacy))
        .layer(CorsLayer::permissive())
        .with_state(state);

    println!("MyContextPort dashboard → {url}");
    println!("Press Ctrl+C to stop.");

    if !no_open {
        let _ = open::that(&url);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Embedded HTML dashboard (no external CDN, no build step)
// ---------------------------------------------------------------------------

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>MyContextPort</title>
<style>
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  :root {
    --bg: #0f0f0f;
    --surface: #1a1a1a;
    --border: #2a2a2a;
    --text: #e8e8e8;
    --muted: #888;
    --accent: #7c6af7;
    --green: #4ade80;
    --yellow: #facc15;
    --orange: #fb923c;
    --red: #f87171;
    --blue: #60a5fa;
  }
  body {
    background: var(--bg);
    color: var(--text);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    font-size: 14px;
    line-height: 1.5;
    min-height: 100vh;
  }
  a { color: var(--accent); text-decoration: none; }
  a:hover { text-decoration: underline; }

  /* Header */
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 24px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .logo { font-size: 15px; font-weight: 600; letter-spacing: -0.3px; }
  .logo span { color: var(--accent); }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: 999px;
    background: #1f1f2e;
    border: 1px solid var(--accent);
    font-size: 12px;
    color: var(--accent);
    font-weight: 500;
  }
  .dot {
    width: 7px; height: 7px;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 2s infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  /* Navigation tabs */
  .tabs {
    display: flex;
    gap: 2px;
    padding: 12px 24px 0;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .tab-btn {
    padding: 8px 16px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--muted);
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    transition: color 0.15s;
    margin-bottom: -1px;
  }
  .tab-btn:hover { color: var(--text); }
  .tab-btn.active { color: var(--accent); border-bottom-color: var(--accent); }

  /* Content */
  .content { padding: 24px; max-width: 1100px; }
  .tab-panel { display: none; }
  .tab-panel.active { display: block; }

  /* Dashboard grid */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 12px;
    margin-bottom: 28px;
  }
  .stat-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 16px;
  }
  .stat-label { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--muted); margin-bottom: 6px; }
  .stat-value { font-size: 28px; font-weight: 700; }

  /* Source bars */
  .section-title { font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--muted); margin-bottom: 12px; }
  .source-bars { display: flex; flex-direction: column; gap: 8px; margin-bottom: 28px; }
  .source-row { display: flex; align-items: center; gap: 10px; }
  .source-name { width: 110px; font-size: 12px; color: var(--text); flex-shrink: 0; }
  .bar-track { flex: 1; height: 8px; background: var(--border); border-radius: 4px; overflow: hidden; }
  .bar-fill { height: 100%; border-radius: 4px; background: var(--accent); transition: width 0.4s ease; }
  .source-count { width: 40px; text-align: right; font-size: 12px; color: var(--muted); }

  /* Recent injections */
  .recent-list { display: flex; flex-direction: column; gap: 8px; }
  .recent-item {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px 14px;
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 12px;
  }
  .recent-model { font-weight: 600; flex: 1; }
  .recent-meta { color: var(--muted); }
  .badge-shared { color: var(--green); }
  .badge-blocked { color: var(--red); }

  /* Empty state */
  .empty {
    text-align: center;
    padding: 48px 24px;
    color: var(--muted);
    font-size: 13px;
  }
  .empty-icon { font-size: 32px; margin-bottom: 12px; }

  /* Table */
  .table-controls {
    display: flex;
    gap: 10px;
    margin-bottom: 14px;
  }
  .search-input, .filter-select {
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 7px 11px;
    border-radius: 6px;
    font-size: 13px;
    outline: none;
  }
  .search-input { flex: 1; }
  .search-input::placeholder { color: var(--muted); }
  .search-input:focus, .filter-select:focus { border-color: var(--accent); }
  .filter-select { background: var(--surface); cursor: pointer; }

  table { width: 100%; border-collapse: collapse; }
  thead th {
    text-align: left;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    padding: 0 12px 10px;
    border-bottom: 1px solid var(--border);
  }
  tbody tr { border-bottom: 1px solid var(--border); }
  tbody tr:hover { background: rgba(255,255,255,0.02); }
  td { padding: 10px 12px; font-size: 12px; vertical-align: top; }
  td.mono { font-family: ui-monospace, monospace; font-size: 11px; color: var(--muted); }
  td.content-cell { max-width: 420px; color: var(--text); }
  .content-preview { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 400px; }
  td.url-cell { max-width: 200px; }
  .url-text { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 190px; display: block; font-size: 11px; color: var(--blue); }

  /* Sensitivity badges */
  .badge {
    display: inline-block;
    padding: 1px 7px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }
  .badge-unknown  { background: #2a2a2a; color: var(--muted); }
  .badge-public   { background: #14532d; color: var(--green); }
  .badge-work     { background: #422006; color: var(--yellow); }
  .badge-personal { background: #431407; color: var(--orange); }
  .badge-health   { background: #450a0a; color: var(--red); }
  .badge-financial{ background: #450a0a; color: var(--red); }

  /* Privacy rules */
  .privacy-meta { margin-bottom: 20px; font-size: 12px; color: var(--muted); }
  .privacy-meta strong { color: var(--text); }
  .rules-list { display: flex; flex-direction: column; gap: 8px; }
  .rule-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 12px 14px;
    display: grid;
    grid-template-columns: 1fr 100px 80px 80px;
    gap: 8px;
    align-items: center;
    font-size: 12px;
  }
  .rule-id { font-weight: 600; color: var(--text); }
  .rule-detail { color: var(--muted); font-size: 11px; margin-top: 2px; }
  .action-allow { color: var(--green); font-weight: 600; }
  .action-block { color: var(--red); font-weight: 600; }
  .action-summarize { color: var(--yellow); font-weight: 600; }
  .scope-pill {
    background: #1a1a2e;
    border: 1px solid #2a2a4a;
    color: var(--blue);
    padding: 2px 7px;
    border-radius: 4px;
    font-size: 10px;
    font-family: ui-monospace, monospace;
  }

  /* Refresh indicator */
  .refresh-bar {
    position: fixed;
    bottom: 16px;
    right: 20px;
    font-size: 11px;
    color: var(--muted);
  }
</style>
</head>
<body>

<div class="header">
  <div class="logo">My<span>Context</span>Port</div>
  <div class="pill">
    <span class="dot"></span>
    <span id="total-pill">— items</span>
  </div>
</div>

<div class="tabs">
  <button class="tab-btn active" data-tab="dashboard">Dashboard</button>
  <button class="tab-btn" data-tab="context">Context Feed</button>
  <button class="tab-btn" data-tab="log">Injection Log</button>
  <button class="tab-btn" data-tab="runs">Collector Runs</button>
  <button class="tab-btn" data-tab="privacy">Privacy Rules</button>
</div>

<div class="content">

  <!-- Dashboard -->
  <div class="tab-panel active" id="tab-dashboard">
    <div class="stats-grid" id="stats-grid">
      <div class="stat-card">
        <div class="stat-label">Total items</div>
        <div class="stat-value" id="stat-total">—</div>
      </div>
      <div class="stat-card">
        <div class="stat-label">Sources</div>
        <div class="stat-value" id="stat-sources">—</div>
      </div>
    </div>

    <div class="section-title">Items by source</div>
    <div class="source-bars" id="source-bars">
      <div class="empty"><div class="empty-icon">📭</div>No data yet. Run <code>mycontextport mcp serve</code> to start collecting.</div>
    </div>

    <div class="section-title" style="margin-top:28px;">Recent injections</div>
    <div class="recent-list" id="recent-injections">
      <div class="empty"><div class="empty-icon">🤖</div>No injections recorded yet. Connect an AI tool via MCP to start.</div>
    </div>
  </div>

  <!-- Context Feed -->
  <div class="tab-panel" id="tab-context">
    <div class="table-controls">
      <input class="search-input" id="ctx-search" type="search" placeholder="Search content…">
      <select class="filter-select" id="ctx-source-filter">
        <option value="">All sources</option>
      </select>
    </div>
    <div id="ctx-table-wrap">
      <div class="empty"><div class="empty-icon">🔍</div>Loading…</div>
    </div>
  </div>

  <!-- Injection Log -->
  <div class="tab-panel" id="tab-log">
    <div id="log-table-wrap">
      <div class="empty"><div class="empty-icon">📋</div>Loading…</div>
    </div>
  </div>

  <!-- Collector Runs -->
  <div class="tab-panel" id="tab-runs">
    <div id="runs-table-wrap">
      <div class="empty"><div class="empty-icon">⏱</div>Loading…</div>
    </div>
  </div>

  <!-- Privacy Rules -->
  <div class="tab-panel" id="tab-privacy">
    <div class="privacy-meta" id="privacy-meta">Loading…</div>
    <div class="rules-list" id="rules-list"></div>
  </div>

</div>

<div class="refresh-bar" id="refresh-bar">—</div>

<script>
// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------
let activeTab = 'dashboard';
let allItems = [];
let allSources = new Set();

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------
document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
    btn.classList.add('active');
    activeTab = btn.dataset.tab;
    document.getElementById('tab-' + activeTab).classList.add('active');
    renderActiveTab();
  });
});

// ---------------------------------------------------------------------------
// Fetch helpers
// ---------------------------------------------------------------------------
async function fetchJSON(url) {
  const r = await fetch(url);
  if (!r.ok) throw new Error(r.statusText);
  return r.json();
}

function fmt(ts) {
  if (!ts) return '—';
  const d = new Date(ts * 1000);
  return d.toLocaleDateString() + ' ' + d.toLocaleTimeString([], {hour:'2-digit',minute:'2-digit'});
}

function sensitivityBadge(s) {
  const cls = 'badge badge-' + (s || 'unknown');
  return `<span class="${cls}">${s || 'unknown'}</span>`;
}

function escHtml(str) {
  return String(str || '')
    .replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')
    .replace(/"/g,'&quot;');
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------
async function loadDashboard() {
  try {
    const [status, log] = await Promise.all([
      fetchJSON('/api/status'),
      fetchJSON('/api/log?limit=5'),
    ]);

    // Pill + stat cards
    document.getElementById('total-pill').textContent = status.items_total + ' items';
    document.getElementById('stat-total').textContent = status.items_total.toLocaleString();
    document.getElementById('stat-sources').textContent = Object.keys(status.items_by_source).length;

    // Source bars
    const entries = Object.entries(status.items_by_source).sort((a,b) => b[1]-a[1]);
    const barsEl = document.getElementById('source-bars');
    if (entries.length === 0) {
      barsEl.innerHTML = '<div class="empty"><div class="empty-icon">📭</div>No data yet. Run <code>mycontextport mcp serve</code> to start collecting.</div>';
    } else {
      const max = entries[0][1];
      barsEl.innerHTML = entries.map(([src, cnt]) => `
        <div class="source-row">
          <div class="source-name">${escHtml(src)}</div>
          <div class="bar-track"><div class="bar-fill" style="width:${Math.round(cnt/max*100)}%"></div></div>
          <div class="source-count">${cnt.toLocaleString()}</div>
        </div>`).join('');
    }

    // Recent injections
    const injEl = document.getElementById('recent-injections');
    if (log.length === 0) {
      injEl.innerHTML = '<div class="empty"><div class="empty-icon">🤖</div>No injections recorded yet. Connect an AI tool via MCP to start.</div>';
    } else {
      injEl.innerHTML = log.map(e => `
        <div class="recent-item">
          <div class="recent-model">${escHtml(e.model)}</div>
          <div class="recent-meta">${fmt(e.injected_at)}</div>
          <div class="recent-meta"><span class="badge-shared">✓ ${e.items_used.length} shared</span></div>
          <div class="recent-meta"><span class="badge-blocked">${e.items_blocked.length > 0 ? '✕ ' + e.items_blocked.length + ' blocked' : ''}</span></div>
        </div>`).join('');
    }
  } catch(e) {
    console.error('Dashboard load failed', e);
  }
}

// ---------------------------------------------------------------------------
// Context Feed
// ---------------------------------------------------------------------------
async function loadContextFeed() {
  try {
    allItems = await fetchJSON('/api/items?limit=200');
    allSources = new Set(allItems.map(i => i.source));

    // Populate source filter
    const sel = document.getElementById('ctx-source-filter');
    sel.innerHTML = '<option value="">All sources</option>' +
      [...allSources].sort().map(s => `<option value="${escHtml(s)}">${escHtml(s)}</option>`).join('');

    renderContextTable();
  } catch(e) {
    document.getElementById('ctx-table-wrap').innerHTML =
      '<div class="empty">Failed to load context items.</div>';
  }
}

function renderContextTable() {
  const query = document.getElementById('ctx-search').value.toLowerCase();
  const srcFilter = document.getElementById('ctx-source-filter').value;

  const filtered = allItems.filter(item => {
    if (srcFilter && item.source !== srcFilter) return false;
    if (query && !item.content.toLowerCase().includes(query)) return false;
    return true;
  });

  const wrap = document.getElementById('ctx-table-wrap');
  if (filtered.length === 0) {
    wrap.innerHTML = '<div class="empty"><div class="empty-icon">🔍</div>No items match your filter.</div>';
    return;
  }

  wrap.innerHTML = `<table>
    <thead><tr>
      <th>Time</th><th>Source</th><th>Sensitivity</th><th>Content</th><th>URL / Path</th>
    </tr></thead>
    <tbody>${filtered.map(item => `
      <tr>
        <td class="mono">${fmt(item.collected_at)}</td>
        <td>${escHtml(item.source)}</td>
        <td>${sensitivityBadge(item.sensitivity)}</td>
        <td class="content-cell"><div class="content-preview" title="${escHtml(item.content)}">${escHtml(item.content)}</div></td>
        <td class="url-cell">${item.url ? `<a class="url-text" href="${escHtml(item.url)}" title="${escHtml(item.url)}" target="_blank" rel="noopener">${escHtml(item.url)}</a>` : '<span style="color:var(--muted)">—</span>'}</td>
      </tr>`).join('')}
    </tbody>
  </table>`;
}

document.getElementById('ctx-search').addEventListener('input', renderContextTable);
document.getElementById('ctx-source-filter').addEventListener('change', renderContextTable);

// ---------------------------------------------------------------------------
// Injection Log
// ---------------------------------------------------------------------------
async function loadLog() {
  try {
    const log = await fetchJSON('/api/log?limit=50');
    const wrap = document.getElementById('log-table-wrap');
    if (log.length === 0) {
      wrap.innerHTML = '<div class="empty"><div class="empty-icon">📋</div>No injections recorded yet.</div>';
      return;
    }
    wrap.innerHTML = `<table>
      <thead><tr>
        <th>Time</th><th>AI Model</th><th>Items Shared</th><th>Items Blocked</th><th>Rules Fired</th>
      </tr></thead>
      <tbody>${log.map(e => `
        <tr>
          <td class="mono">${fmt(e.injected_at)}</td>
          <td>${escHtml(e.model)}</td>
          <td style="color:var(--green)">${e.items_used.length}</td>
          <td style="color:${e.items_blocked.length > 0 ? 'var(--red)' : 'var(--muted)'}">${e.items_blocked.length}</td>
          <td class="mono">${e.rules_applied.length > 0 ? escHtml(e.rules_applied.join(', ')) : '<span style="color:var(--muted)">—</span>'}</td>
        </tr>`).join('')}
      </tbody>
    </table>`;
  } catch(e) {
    document.getElementById('log-table-wrap').innerHTML =
      '<div class="empty">Failed to load injection log.</div>';
  }
}

// ---------------------------------------------------------------------------
// Privacy Rules
// ---------------------------------------------------------------------------
async function loadPrivacy() {
  try {
    const data = await fetchJSON('/api/privacy');
    const meta = document.getElementById('privacy-meta');
    meta.innerHTML = `Unknown client policy: <strong>${escHtml(data.unknown_client_policy)}</strong> &nbsp;·&nbsp; ${data.rules.length} rule${data.rules.length !== 1 ? 's' : ''} loaded`;

    const list = document.getElementById('rules-list');
    if (data.rules.length === 0) {
      list.innerHTML = '<div class="empty"><div class="empty-icon">🔓</div>No privacy rules configured. All context is shared with all AI tools.<br><br>Create <code>privacy.toml</code> in your store directory to add rules.</div>';
      return;
    }
    list.innerHTML = data.rules.map(r => {
      const actionCls = 'action-' + (r.action || 'allow');
      return `<div class="rule-card">
        <div>
          <div class="rule-id">${escHtml(r.id)}</div>
          <div class="rule-detail">${escHtml(r.rule_type)} · pattern: <code>${escHtml(r.pattern)}</code></div>
        </div>
        <span class="${actionCls}">${(r.action || 'allow').toUpperCase()}</span>
        <span>${r.model_scope ? `<span class="scope-pill">${escHtml(r.model_scope)}</span>` : '<span style="color:var(--muted);font-size:11px">all models</span>'}</span>
        <span></span>
      </div>`;
    }).join('');
  } catch(e) {
    document.getElementById('privacy-meta').textContent = 'Failed to load privacy rules.';
  }
}

// ---------------------------------------------------------------------------
// Collector Runs
// ---------------------------------------------------------------------------
async function loadRuns() {
  try {
    const runs = await fetchJSON('/api/runs?limit=50');
    const wrap = document.getElementById('runs-table-wrap');
    if (runs.length === 0) {
      wrap.innerHTML = '<div class="empty"><div class="empty-icon">⏱</div>No collector runs recorded yet. Start <code>mycontextport mcp serve</code>.</div>';
      return;
    }
    wrap.innerHTML = `<table>
      <thead><tr>
        <th>Collector</th><th>Started</th><th>Duration</th><th>Found</th><th>Inserted</th><th>Status</th>
      </tr></thead>
      <tbody>${runs.map(r => {
        const dur = (r.finished_at && r.started_at)
          ? ((r.finished_at - r.started_at) + 's')
          : '…';
        const status = r.error
          ? `<span style="color:var(--red)">${escHtml(r.error)}</span>`
          : r.finished_at
            ? '<span style="color:var(--green)">ok</span>'
            : '<span style="color:var(--yellow)">running</span>';
        return `<tr>
          <td>${escHtml(r.collector)}</td>
          <td class="mono">${fmt(r.started_at)}</td>
          <td class="mono">${dur}</td>
          <td>${r.items_found ?? '—'}</td>
          <td>${r.items_inserted ?? '—'}</td>
          <td>${status}</td>
        </tr>`;
      }).join('')}
      </tbody>
    </table>`;
  } catch(e) {
    document.getElementById('runs-table-wrap').innerHTML =
      '<div class="empty">Failed to load collector runs.</div>';
  }
}

// ---------------------------------------------------------------------------
// Render active tab on demand
// ---------------------------------------------------------------------------
function renderActiveTab() {
  if (activeTab === 'dashboard') loadDashboard();
  else if (activeTab === 'context') loadContextFeed();
  else if (activeTab === 'log') loadLog();
  else if (activeTab === 'runs') loadRuns();
  else if (activeTab === 'privacy') loadPrivacy();
}

// ---------------------------------------------------------------------------
// Auto-refresh every 30 s; always refresh status pill
// ---------------------------------------------------------------------------
let refreshTimer;
async function refreshStatus() {
  try {
    const s = await fetchJSON('/api/status');
    document.getElementById('total-pill').textContent = s.items_total + ' items';
  } catch(_) {}
  const now = new Date();
  document.getElementById('refresh-bar').textContent =
    'Last updated ' + now.toLocaleTimeString([], {hour:'2-digit',minute:'2-digit',second:'2-digit'});
}

function scheduleRefresh() {
  clearTimeout(refreshTimer);
  refreshTimer = setTimeout(async () => {
    await refreshStatus();
    renderActiveTab();
    scheduleRefresh();
  }, 30_000);
}

// ---------------------------------------------------------------------------
// Boot
// ---------------------------------------------------------------------------
loadDashboard();
refreshStatus();
scheduleRefresh();
</script>
</body>
</html>"#;
