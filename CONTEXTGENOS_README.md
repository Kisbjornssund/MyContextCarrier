# ContextGenOS

### The Universal Personal Memory Layer for AI

> *Your context. Your data. Every AI, everywhere.*

---

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Status: Pre-Alpha](https://img.shields.io/badge/Status-Pre--Alpha-orange.svg)]()
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green.svg)]()
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)]()

---

## What Is ContextGenOS?

ContextGenOS is an open source, **locally-run personal AI memory system**: a middleware layer that sits between *you* and every AI model or application you use.

It captures, structures, and selectively injects your personal context into any AI interaction. Your work history, ongoing projects, personal preferences, relationships, habits, and knowledge, available to Claude, GPT, Gemini, Mistral, Llama, or any AI tool you choose. Privately. Locally. Permanently under your control.

You stop re-explaining yourself to every AI you open. Your AI tools stop being amnesiac strangers and start behaving like colleagues who know you.

---

## The Problem

Every AI you use starts from zero.

You open Claude and explain your project. You open Cursor and explain your codebase conventions. You open Perplexity and explain what kind of answer you want. You open a new chat and start over entirely.

Your context, who you are, what you're working on, how you think, what you've already tried, is locked inside individual apps, siloed behind proprietary walls, or simply lost. The AI revolution promised intelligence that *knows you*. What we got instead were brilliant strangers who forget you the moment the window closes.

The solutions that exist are worse than the problem. Cloud memory services harvest your most intimate professional and personal details to feed their own models. App-specific memory (like ChatGPT's) only works in one place. Manual note systems require you to do work the AI should be doing. Nothing is open. Nothing is local. Nothing is portable. Nothing is yours.

ContextGenOS is the answer that should have been built first.

---

## Core Principles

**1. Local by default, always.**
Your data never leaves your machine unless you explicitly choose otherwise. No telemetry. No phone-home. No "we may use your data to improve our models." The entire system runs on your hardware.

**2. Model-agnostic.**
ContextGenOS is not loyal to any AI provider. It speaks to every model through a universal context API and native MCP (Model Context Protocol) support. Use it with the model you prefer today, and a different one tomorrow.

**3. Context compounds.**
The longer you use ContextGenOS, the more valuable it becomes. Every conversation, every project, every preference enriches the layer that makes every future AI interaction smarter. This is the flywheel that cloud AI wants to own. ContextGenOS gives it back to you.

**4. Contributor-native architecture.**
Every data source you can imagine, email, calendar, browser history, GitHub, Notion, health data, financial records, is a plugin. The architecture is modular by design so that every developer can contribute the domain they know best.

**5. Radical transparency.**
You can inspect, edit, export, or delete any piece of context ContextGenOS holds. You can see exactly what context is injected into any given prompt. No black boxes. No opaque embeddings you can't audit.

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   DATA SOURCES                      │
│  Browser · Email · Calendar · GitHub · Notes · ...  │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│              CONTEXT COLLECTORS                     │
│         Modular plugin layer (contribute here)      │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│               CONTEXTGENOS CORE                        │
│   DuckDB (structured) + Qdrant (vector) local store │
│   Context graph · Relevance ranking · Privacy rules │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│          UNIVERSAL CONTEXT API / MCP SERVER         │
│     REST · gRPC · MCP Protocol · Python SDK         │
└────────┬───────────────────────────────┬────────────┘
         │                               │
         ▼                               ▼
┌────────────────┐             ┌─────────────────────┐
│  Claude / GPT  │             │  Cursor / Obsidian  │
│  Gemini / Llama│             │  VS Code / Any App  │
└────────────────┘             └─────────────────────┘
```

**Core stack:**
- **Rust**: core daemon for performance and memory safety
- **DuckDB**: local structured data store
- **Qdrant**: local vector search (embedded mode)
- **MCP Protocol**: native AI model integration standard
- **Python SDK**: for contributor accessibility and plugin development

---

## Features (Roadmap)

### v0.1: Foundation
- [ ] Local context store (DuckDB + Qdrant embedded)
- [ ] MCP server with context injection API
- [ ] Context collector: Browser history (Chrome, Firefox, Safari)
- [ ] Context collector: Calendar (Google Calendar, iCal)
- [ ] Context collector: Notes (Markdown files, Obsidian vault)
- [ ] Basic web UI for context inspection and editing
- [ ] Python SDK

### v0.2: Integration
- [ ] Context collector: GitHub (repos, PRs, issues, commit history)
- [ ] Context collector: Email (local IMAP, metadata-only mode)
- [ ] Claude, GPT, Gemini native integrations
- [ ] Privacy rules engine (exclude domains, topics, time ranges)
- [ ] Context diff viewer (what changed, what was injected)

### v0.3: Intelligence
- [ ] Context relevance scoring (inject only what matters for this prompt)
- [ ] Automatic context summarization and compression
- [ ] Cross-session memory graph
- [ ] Context collector: Slack / Discord
- [ ] Context collector: Linear / Jira
- [ ] Export / import (full data portability)

### v1.0: Platform
- [ ] Plugin marketplace
- [ ] Optional encrypted cloud sync (user-hosted)
- [ ] Multi-device context federation
- [ ] Team context sharing (opt-in, scoped)

---

## Getting Started

> ⚠️ ContextGenOS is pre-alpha. The API is unstable. Here be dragons. We love early contributors.

```bash
# Install
curl -fsSL https://contextgenos.dev/install.sh | sh

# Initialize your context store
contextgenos init

# Start the local daemon
contextgenos start

# Add your first context source
contextgenos collector add browser
contextgenos collector add calendar

# Check what ContextGenOS knows about you
contextgenos inspect

# Connect to Claude via MCP
contextgenos mcp serve --port 8765
```

Then in your Claude (or any MCP-compatible) configuration:
```json
{
  "mcp_servers": [
    {
      "name": "contextgenos",
      "url": "http://localhost:8765/mcp"
    }
  ]
}
```

From this point forward, every AI conversation you have is aware of your world.

---

## Contributing

ContextGenOS lives or dies by its contributor community. The plugin architecture is designed so you can contribute something meaningful in an afternoon.

**Ways to contribute:**

**Context Collectors:** Build a plugin for any data source you use. If you live in Notion, Roam, Todoist, Spotify, Garmin, or anywhere else, you have a collector to build. See `docs/collector-spec.md`.

**Core:** Rust and systems experience needed. Performance, privacy, storage, and the context graph are all active areas.

**Integrations:** Connect ContextGenOS to AI tools and IDEs. VS Code extension, JetBrains plugin, browser extension, all open.

**Documentation:** Excellent technical writing is one of the highest-leverage contributions an open source project can receive.

**Design:** The UI for inspecting and managing your personal context should be beautiful and trustworthy. Design contributions welcome.

See `CONTRIBUTING.md` to get started. Join the conversation on Discord.

---

## Philosophy

We believe that personal context, the accumulated record of how you think, what you work on, and who you are professionally and intellectually, is among the most valuable and intimate data a person generates. It should not be owned by any company. It should not be used to train any model without explicit consent. It should not be locked inside any application.

ContextGenOS is infrastructure for AI that respects human sovereignty. It is not a product. It is a public good.

---

## License

MIT. Use it, fork it, build on it, sell products with it.

---

## Community

- **Discord:** discord.gg/contextgenos
- **GitHub Discussions:** github.com/contextgenos/contextgenos/discussions
- **Twitter/X:** @contextgenosdev
- **Website:** contextgenos.dev

---

*Built in public. Owned by no one. Useful to everyone.*

---

---

# THE CONTEXTGENOS MANIFESTO

## *For the Right to Be Known Without Being Watched*

---

Something quietly broke in the AI revolution.

The models got extraordinary. The interfaces got beautiful. The demos got genuinely magical. And yet, every single day, billions of people open an AI tool and begin the same ritual: re-explaining themselves.

Who they are. What project they're working on. What they've already tried. What their preferences are. How they like to think. What they already know. What they don't need explained.

Over and over. App by app. Session by session. The AI that is supposed to be intelligent has no memory. The AI that is supposed to help you has no idea who you are.

This is not an accident.

---

### The Architecture of Forgetting

The leading AI systems are built to be amnesiac by design — not because memory is technically hard, but because memory is commercially valuable. Your context, accumulated and stored in their systems, is not a service they offer you. It is an asset they extract from you.

When a company holds your context, they hold your professional history, your intellectual patterns, your projects, your relationships, your ambitions, your fears. They know what you're building before you ship it. They know what problems you haven't solved. They know what questions reveal the edges of your understanding.

This is extraordinarily intimate data. And the business model of cloud AI is to accumulate as much of it as possible, use it to improve their models, and lock you into their ecosystem because leaving means losing the intelligence that has come to know you.

The alternatives offered to us are not real alternatives. "Bring your own memory" systems that upload your data to yet another cloud. App-specific memory that works in one product and nowhere else. Local tools that require you to manually curate everything the AI should be learning automatically. Enterprise solutions that cost thousands of dollars a month and still don't give you ownership.

We are being offered the appearance of sovereignty while the substance is extracted.

---

### What We Actually Want

We want AI that knows us. This is a reasonable desire. It is not a luxury or an advanced feature. It is the basic premise of working with an intelligent system over time.

We want AI that remembers our projects, understands our work style, recalls what we've tried, knows what we know, and adapts to how we think. We want AI that gets better at working with us the longer we use it — not because it has mined us for training data, but because it has genuinely learned our context.

We want this to work across every AI tool we use, not just the one whose parent company owns our memory.

We want to be able to inspect what is known about us. To correct it. To delete it. To export it. To move it. To understand exactly what context is being used in any given conversation.

We want to own this layer of our digital lives.

None of this is technologically impossible. Every piece of it is buildable today. What has been missing is not capability — it is will. Specifically, the will to build it in the open, to keep it local, and to give away the thing that cloud AI companies are determined to monetize.

---

### The Moment We Are In

Two things are true right now that were not true two years ago.

First, the hardware exists. Apple Silicon, consumer Nvidia GPUs, and the rapid commoditization of inference mean that running a sophisticated, always-on local AI middleware layer is within reach of any laptop manufactured in the last three years. Local AI is no longer a hobbyist compromise. It is a serious option.

Second, the protocol exists. MCP — the Model Context Protocol — has emerged as a genuine standard for how AI models receive context from external systems. For the first time, there is a common language that a universal context layer can speak to connect with any compliant AI tool. The infrastructure moment has arrived.

These two developments together mean that ContextGenOS is not a speculative project. It is an immediately buildable one. The technical foundation is there. The timing is right. The only question is whether the open source community will build it before the incumbents make it unnecessary by locking the context layer inside their walled gardens permanently.

We think the open source community will build it. We think that because this community has built every other layer of the infrastructure stack that the world depends on. The operating system. The web server. The database. The programming language. The container runtime. The AI framework. Every time a critical piece of infrastructure emerged, the open source community eventually built the version that respected users and didn't extract rent from them.

Context is the next layer. ContextGenOS is our answer.

---

### What We Are Building

We are building infrastructure. Not a product. Not a startup. Not a platform designed to maximize engagement or extract value. Infrastructure.

Infrastructure that runs locally and keeps your data on your machine. Infrastructure that speaks to every AI model through open standards. Infrastructure that gets more valuable the longer you use it because it compounds your context, not because it locks you in. Infrastructure that any developer can inspect, modify, extend, or fork.

We are building the layer that should have been built at the beginning of the AI revolution — the layer that gives individuals the same advantage that enterprises currently pay fortunes for: AI that genuinely knows its user.

We are building this in public, with a license that keeps it free forever, with an architecture designed explicitly to make contribution easy. Every data source you can imagine is a plugin. Every AI tool you use is an integration target. Every person who has ever been frustrated by explaining themselves to an AI for the hundredth time has a reason to contribute.

---

### Our Commitments

**To users:** Your context will never leave your machine without your explicit command. You will always be able to see exactly what ContextGenOS knows about you and exactly what context was injected into any conversation. You will be able to delete everything. You will be able to export everything. These are not features we might add. They are promises that are encoded in the architecture.

**To contributors:** The plugin system is designed so that meaningful contribution is possible in an afternoon. The codebase will be documented as if documentation is as important as the code itself — because it is. Decisions about architecture and direction will be made in public. No contributor will be surprised by a pivot that serves a business model they didn't sign up for, because there is no business model. There is only the software.

**To the ecosystem:** ContextGenOS will support every open standard and resist every proprietary one. We will implement MCP because it is open. We will not implement closed context protocols designed to lock users into specific AI platforms. We will actively work to make ContextGenOS unnecessary by advocating for privacy-respecting context standards to be adopted natively by AI tools — and we will celebrate if we succeed, because the goal is not for ContextGenOS to exist forever, but for users to be in control.

**To the idea:** We are building this because we believe that the right to be known without being watched is worth defending. That personal context is intimate enough to deserve sovereignty. That the accumulation of who you are intellectually and professionally should not be a revenue stream for someone else. This manifesto is a record of why we started. If we ever drift from it, point back here.

---

### An Invitation

If you have been frustrated by explaining yourself to AI over and over, this is your project.

If you have been uneasy about what cloud AI services know about you and what they do with it, this is your project.

If you are a developer who has a data source you live in every day and wish your AI tools understood, this is your project.

If you believe that open source infrastructure is how the world gets the tools it deserves rather than the tools that maximize shareholder return, this is your project.

The first version is already being built. The architecture is open. The issues are labeled. The Discord is live.

Come build with us.

---

*ContextGenOS — Local by default. Open by design. Yours, permanently.*

*github.com/contextgenos/contextgenos*
