# MyContextCarrier

<img width="2848" height="1600" alt="MyContextCarrier" src="https://github.com/user-attachments/assets/74d19fca-3df0-44ab-968e-70bbdf6b0267" />

### The Universal, Portable, Private and Persistent AI Memory You Own.

MyContextCarrier started as a personal project to address the annoyance of porting context between different AI tools (including agentic workflows) and having to re-explain everything, every time. Context windows are limited and exclusive: you can't carry them with you wherever you go.

It grew into a privacy-gated MCP server that collects personal context locally, enforces user-defined sensitivity rules per model, and injects the right context into any MCP-compatible AI client.

> *Your context. Your data. Every AI, everywhere.*

---

[![CI](https://github.com/Kisbjornssund/MyContextCarrier/actions/workflows/ci.yml/badge.svg)](https://github.com/Kisbjornssund/MyContextCarrier/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Status: Pre-Alpha](https://img.shields.io/badge/Status-Pre--Alpha-orange.svg)]()
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green.svg)]()
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Discord](https://img.shields.io/badge/Discord-Join%20us-7289da.svg)](https://discord.gg/NvqtCBRr)

---

<!-- DEMO GIF PLACEHOLDER -->
<!-- Replace with: terminal recording showing `mycontextport mcp serve` running, -->
<!-- then an AI tool demonstrating awareness of context it was never told.     -->
<!-- This is the single most important asset in this README.                   -->
<!-- ![MyContextCarrier Demo](docs/assets/demo.gif)                            -->

---

## The Problem No One Has Solved

**Part 1: The repetition tax.** Every AI tool you open starts from zero. You explain your project to one assistant. You explain your codebase to another. You explain your preferences to a third. Every session. Every app. Every time. The AI revolution promised intelligence that *knows you*. What we got were brilliant strangers with no memory.

**Part 2: The privacy trap.** Every solution that does remember you — ChatGPT Memory, cloud-based project tools, Mem.ai — solves repetition by uploading your most sensitive data to servers you don't control. Your browser history, calendar, work notes, and emails are transmitted to third parties. Potentially used to train models. Potentially exposed in breaches. Subject to legal demands you'll never see.

**Today's impossible choice:**
- Option A: Have AI that knows you, but surrender your personal data to corporate clouds
- Option B: Keep your data private, but re-explain yourself every single session

**MyContextCarrier solves both.** Your context is captured and stored locally. You decide exactly what gets shared with which AI tool — and nothing leaves your device unless you choose to inject it.

---

## Privacy Is a Spectrum, Not a Mandate

| Mode | What it does | Who it's for |
|------|-------------|--------------|
| **Maximum privacy** | Context never leaves your device. Cloud AI receives only the injected snippet, not your raw data | Privacy-first users |
| **Selective sharing** | Per-model rules: full context to local Ollama, work-only to cloud tools, nothing to others | Most users |
| **Transparent** | Full audit log of every injection: see exactly what data each model received, with timing traces | Auditors, the curious |
| **Export / delete** | Full data portability at any time. Delete everything. It's yours. | Everyone |

---

## Get Started

```bash
# Build from source
git clone https://github.com/Kisbjornssund/MyContextCarrier
cd MyContextCarrier
cargo build --release

# Start collecting context (runs in background)
./target/release/mycontextport mcp serve
```

Then add this to your Claude Desktop config (`claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "mycontextcarrier": {
      "command": "/path/to/mycontextport",
      "args": ["mcp", "serve"]
    }
  }
}
```

Open Claude. Ask: *"What have I been working on?"*

It already knows.

> **Note:** MyContextCarrier is pre-alpha. The API is unstable. We love early contributors. [See how to help.](CONTRIBUTING.md)

---

## How It Works

```
┌─────────────────────────────────────────────────────┐
│                   DATA SOURCES                      │
│  Browser · Calendar · Notes · Email · VS Code · ... │
└────────────────────┬────────────────────────────────┘
                     │ Context Collectors (Python plugins)
                     ▼
┌─────────────────────────────────────────────────────┐
│           MYCONTEXTCARRIER CORE (Rust)              │
│          DuckDB local store · Entity graph          │
│   Per-collector scheduler · Memory threads          │
│   Privacy rules · Guardrails · Audit traces         │
└────────────────────┬────────────────────────────────┘
                     │ Privacy Rules Engine
                     ▼
┌─────────────────────────────────────────────────────┐
│          UNIVERSAL CONTEXT API / MCP SERVER         │
│         MCP Protocol · REST dashboard · Python SDK  │
└────────┬───────────────────────────────┬────────────┘
         │                               │
         ▼                               ▼
┌────────────────┐             ┌─────────────────────┐
│  Claude / GPT  │             │  Cursor / Obsidian  │
│  Gemini / Llama│             │  Zed / Any MCP app  │
└────────────────┘             └─────────────────────┘
```

**Core stack:** Rust daemon · DuckDB · MCP Protocol · Python SDK

---

## Key Features

**Collectors** — Python plugins that pull context from local data sources. Each runs on its own schedule with an independent timeout. Built-in: browser history, notes, calendar, email, VS Code activity.

**Memory threads** — Group context by project or topic. Threads carry custom instructions that are injected alongside their items. Per-thread `agent_scope` controls which MCP clients can access them.

**Knowledge graph** — Rule-based entity extraction (people, tools, projects, concepts) links items semantically. The `search_context` MCP tool queries the graph; falls back to recency if the graph is empty.

**Privacy engine** — Sensitivity tiers (Public → Health/Financial) with per-model allow/block rules evaluated before every injection. Full audit log with timing traces per call.

**Guardrails** — Separate write-back gate: `append_context` and `create_thread` evaluate `[[guardrails]]` rules before storing anything. Actions: Block, Redact, RequireConfirmation.

---

## MCP Tools

| Tool | What it does |
|------|-------------|
| `get_context` | Recent items, privacy-filtered |
| `search_context` | Graph-aware keyword search |
| `append_context` | Write back from agent (guardrail-gated) |
| `list_sources` | Source names + item counts |
| `list_threads` | Accessible memory threads |
| `get_thread_context` | Thread instructions + items |
| `create_thread` | New project-scoped thread |
| `assign_to_thread` | Link an item to a thread |

---

## Ecosystem & Integrations

### Works with Claude Desktop

```json
{
  "mcpServers": {
    "mycontextcarrier": {
      "command": "/path/to/mycontextport",
      "args": ["mcp", "serve"]
    }
  }
}
```

### Works with Cursor, Zed, and any MCP client

Any editor or AI tool with MCP support connects the same way. The same context, the same privacy rules, everywhere you work.

### Works with Ollama

Sensitive context tiers (Health, Financial, Personal) are never routed to cloud models. Pair with a local Ollama model for fully air-gapped operation.

---

## Supported Collectors

| Collector | Status | Platform |
|-----------|--------|----------|
| Browser history (Chrome, Firefox) | ✅ shipped | macOS, Linux |
| Markdown / Obsidian notes | ✅ shipped | All |
| Google Calendar / iCal | ✅ shipped | All |
| VS Code activity | ✅ shipped | macOS, Linux |
| Email (local mbox, metadata-only) | ✅ shipped | All |
| GitHub (repos, PRs, issues) | planned | All |
| Slack / Discord | planned | All |
| Linear / Jira | planned | All |
| **Your collector here** | [Contribute →](CONTRIBUTING.md) | |

---

## Roadmap

| Version | Status | What shipped |
|---------|--------|-------------|
| **v0.1** | ✅ Done | Browser + notes collectors · MCP server · CLI · Python SDK |
| **v0.2** | ✅ Done | Calendar collector · Python bridge · `collector` CLI subcommand · `append_context` + `list_sources` MCP tools |
| **v0.3** | ✅ Done | Per-collector scheduler · Memory threads · Thread MCP tools · Audit traces · Guardrails engine |
| **v0.4** | ✅ Done | Email collector · VS Code collector · Concurrent multi-source runs |
| **v0.5** | ✅ Done | Knowledge graph · Entity extraction · `search_context` MCP tool |
| **v0.6** | Planned | GitHub collector · Windows support · TUI dashboard |
| **v1.0** | Planned | Stable API · Security audit · Optional user-hosted sync |

---

## Contributing

The plugin architecture is designed so you can ship something meaningful in an afternoon.

**Build a collector** (Python, any experience level):
```python
from mycontextport import BaseCollector, ContextItem, CollectorHealth

class MyCollector(BaseCollector):
    async def collect(self) -> list[ContextItem]:
        return [ContextItem(content="...", source="my-tool")]

    async def health_check(self) -> CollectorHealth:
        return CollectorHealth(healthy=True, message="ready")
```

Scaffold one instantly:
```bash
mycontextport dev new-collector my-tool
```

**Other ways to contribute:**

| Domain | Prerequisites | Effort |
|--------|--------------|--------|
| Context Collectors | Python basics | Afternoon |
| Core Daemon | Rust, async/tokio | Days |
| MCP Tools | Rust | Half-day |
| Documentation | Writing | 1–3 hours |

→ [Read CONTRIBUTING.md](CONTRIBUTING.md)
→ [Join Discord](https://discord.gg/NvqtCBRr) — `#collectors-dev` is where the action is

---

## Philosophy

Personal context — the accumulated record of how you think, what you work on, who you are professionally and intellectually — is among the most intimate data you generate. It should not be owned by any company. It should not be used to train models without explicit consent. It should not be locked inside any application.

MyContextCarrier is infrastructure for AI that respects human sovereignty. It is not a product. It is a public good.

[Read the full Manifesto →](MANIFESTO.md)

---

## Community

- **Discord:** [discord.gg/NvqtCBRr](https://discord.gg/NvqtCBRr)
- **GitHub Discussions:** [Discussions](https://github.com/Kisbjornssund/MyContextCarrier/discussions)
- **Twitter/X:** [@mycontextportdev](https://x.com/mycontextportdev)

---

## License

MIT. Use it, fork it, build on it, sell products with it.

---

*Built in public. Owned by no one. Useful to everyone.*
