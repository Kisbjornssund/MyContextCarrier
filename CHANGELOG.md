# Changelog

All notable changes to MyContextCarrier are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
MyContextCarrier follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned
- GitHub collector (repos, PRs, issues, commits)
- Windows support
- TUI dashboard
- Cross-device sync (user-hosted encrypted storage)
- Stable API + security audit (v1.0)

---

## [0.5.0] — Knowledge Graph

### Added
- `mycontextport-graph` crate: rule-based entity extraction (people, projects, tools, concepts)
- `GraphIndexer`: indexes context items into `graph_nodes` / `graph_edges` tables; startup indexing via `spawn_blocking`
- `search_context` MCP tool: graph-aware keyword search, falls back to recency if graph is empty
- `@mention` → person, markdown headings → project, 50+ known tool names, email sender metadata, VSCode workspace URI

---

## [0.4.0] — New Collectors + Concurrent Scheduling

### Added
- Email collector: reads local mbox files via stdlib; work/personal sensitivity classification by sender domain; body opt-in via `include_body` config; stable IDs from Message-ID hash
- VSCode collector: reads `state.vscdb` for recent workspaces and `History/` for recently edited files; supports VSCode, VSCodium, Insiders on macOS/Linux/Windows
- Concurrent multi-source runs: `Scheduler` uses `tokio::JoinSet` so all collectors run in parallel with independent timeouts

---

## [0.3.0] — Memory Threads + Observability

### Added
- Per-collector scheduler: each collector runs in its own `tokio::spawn` task with independent interval and 60s timeout
- Memory threads: `threads` table with `agent_scope` glob, custom instructions, archive support
- MCP tools: `list_threads`, `get_thread_context`, `create_thread`, `assign_to_thread`
- Audit traces: `TraceStep` records timing for fetch / privacy_eval / inject phases per MCP call
- `log_injection_full`: stores `thread_id` and `Vec<TraceStep>` alongside injection log entries
- Guardrails engine: `Block`, `Redact`, `RequireConfirmation` actions on `append_context` and `create_thread`; loads from `[[guardrails]]` in `privacy.toml`
- Dashboard: `/api/runs`, `/api/log/:id/trace` endpoints; Collector Runs tab

---

## [0.2.0] — Live Data + CLI Completeness

### Added
- Calendar collector: parses `.ics` files via `icalendar`; handles all-day and datetime events; stable IDs from UID hash; extracts organiser and attendees
- Python subprocess bridge (`PythonCollector`): Rust spawns Python scripts with JSON config on stdin; `--health` flag support
- `runner.py` SDK entrypoint: used by all collector `__main__.py` files
- CLI `collector` subcommand: `list`, `add`, `remove`, `health`
- CLI `dev new-collector <name>`: scaffolds `collector.py` + `__main__.py`
- MCP tools: `append_context` (guardrail-gated write-back), `list_sources`
- Store migrations: `thread_id` column on `context_items`, `trace` column on `injection_log`
- `collectors.toml` format for registering Python collectors with paths and intervals

---

## [0.1.0] — Initial Release

### Added
- Core Rust daemon (`mycontextport-core`)
- Local context store (DuckDB, embedded, no separate server)
- Shell history collector
- Browser history collector (Chrome, Firefox, macOS + Linux)
- Markdown / Obsidian notes collector
- MCP server (`mycontextport mcp serve`) — stdio transport, JSON-RPC 2.0
- Privacy rules engine: sensitivity tiers, per-model allow/block rules, `unknown_client` policy
- CLI: `mcp serve`, `status`, `inspect`, `log`
- Python SDK (`pip install mycontextport`), `BaseCollector` interface
- Web dashboard (`mycontextport mcp serve --dashboard`)

---

[Unreleased]: https://github.com/Kisbjornssund/MyContextCarrier/compare/HEAD...HEAD
