---
sidebar_position: 1
title: Quickstart
description: Install ContextGenOS and connect it to Claude in under 10 minutes.
---

# Quickstart

**Time to complete: 10 minutes**

By the end of this guide, you will have ContextGenOS running locally, connected to Claude via MCP, and Claude will already know what you have been working on, without you typing a word of context.

---

## Step 1: Install

```bash
curl -fsSL https://contextgenos.dev/install.sh | sh
```

Or with Docker (no Rust installation required):

```bash
docker pull ghcr.io/kisbjornssund/contextgenos:latest
```

Verify the install:

```bash
contextgenos --version
```

---

## Step 2: Initialize

Run the interactive setup wizard:

```bash
contextgenos init --wizard
```

The wizard asks four questions:
1. Where are your notes? (directory path or skip)
2. Which browser do you use? (Chrome, Firefox, or skip)
3. Which AI tools do you use? (for MCP config generation)
4. Confirm the privacy defaults

This takes about 3 minutes and requires no manual config file editing.

---

## Step 3: Connect to Claude

Add the following to your Claude desktop configuration file.

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`

**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

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

Start the MCP server:

```bash
contextgenos mcp serve --port 8765
```

---

## Step 4: Verify it works

Open Claude. Type:

> What have I been working on recently?

Claude should respond with context drawn from your notes and browser history, without you pasting anything.

If Claude does not show context, run:

```bash
contextgenos status
contextgenos inspect --limit 10
```

This shows what ContextGenOS has collected and whether the MCP server is reachable.

---

## What to do next

- [Inspect your context store](../cli/inspect.md): see exactly what ContextGenOS knows
- [Configure privacy rules](../privacy/rules.md): control what goes to which model
- [Build a collector](../collectors/writing-a-collector.md): add support for a tool you use
- [Connect another AI tool](../mcp/integrations.md): Ollama, GPT-4, Cursor
- [Join Discord](https://discord.gg/NvqtCBRr): `#show-and-tell` for setup screenshots
