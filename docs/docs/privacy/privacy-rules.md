---
sidebar_position: 1
---

# Privacy Rules

MyContextPort evaluates a privacy rules file before injecting any context into an AI model. Rules are evaluated in order — the first matching rule wins. If no rule matches, the fallback depends on whether the connecting client is *known* (see [Unknown client policy](#unknown-client-policy) below).

## Config file location

Place your rules at:

```
~/.local/share/MyContextPort/privacy.toml   # Linux
~/Library/Application Support/MyContextPort/privacy.toml  # macOS
```

The file is created (empty = allow-all) the first time you run `mycontextport init`.

---

## Defaults

Optional global settings placed at the top of the file:

```toml
[defaults]
unknown_client = "deny"   # allow | deny  (default: allow)
```

| Setting | Values | Default | Effect |
|---|---|---|---|
| `unknown_client` | `allow` / `deny` | `allow` | What to do when a connecting client's name matches no `model_scope` in the ruleset |

See [Unknown client policy](#unknown-client-policy) for details.

---

## Rule structure

Each rule is a `[[rules]]` table entry:

```toml
[[rules]]
id = "unique-rule-id"         # required, used in audit log
rule_type = "sensitivity"     # see Rule Types below
pattern = "health"            # glob pattern to match against
action = "block"              # allow | block | summarize
model_scope = "claude*"       # optional: only applies to models matching this glob
```

---

## Rule types

| `rule_type` | Matches against |
|---|---|
| `sensitivity` | The item's sensitivity tier: `public`, `work`, `personal`, `health`, `financial`, `unknown` |
| `source` | The collector that produced the item: `shell_history`, `browser`, `notes`, etc. |
| `url_pattern` | The URL or file path associated with the item |
| `content_pattern` | A keyword found anywhere in the item's text content |
| `backend_scope` | Reserved for the agent layer (backend type selection) |

---

## Actions

| `action` | Effect |
|---|---|
| `allow` | Inject the item (default if no rule matches) |
| `block` | Suppress the item entirely |
| `summarize` | Currently treated as `allow`; future: inject a truncated summary |

---

## Example: protect sensitive data from cloud models

```toml
# Block health and financial data from going to any Claude model
[[rules]]
id = "block-health-cloud"
rule_type = "sensitivity"
pattern = "health"
action = "block"
model_scope = "claude*"

[[rules]]
id = "block-financial-cloud"
rule_type = "sensitivity"
pattern = "financial"
action = "block"
model_scope = "claude*"

[[rules]]
id = "block-personal-cloud"
rule_type = "sensitivity"
pattern = "personal"
action = "block"
model_scope = "claude*"

# Ollama (local) gets everything
# (no rules needed — allow-all is the default)
```

---

## Example: block a specific source

```toml
[[rules]]
id = "block-browser-from-gpt"
rule_type = "source"
pattern = "browser"
action = "block"
model_scope = "gpt*"
```

---

## Example: content-based redaction

```toml
# Block any item mentioning salary figures from reaching cloud models
[[rules]]
id = "block-salary-content"
rule_type = "content_pattern"
pattern = "salary"
action = "block"
model_scope = "claude*"
```

---

## Unknown client policy

A client is **known** if its `clientInfo.name` (sent during the MCP `initialize` handshake) matches the `model_scope` of at least one rule. A client is **unknown** if it matches no `model_scope` anywhere in the ruleset.

Rules without a `model_scope` apply to all clients but do not make any client "known".

| Client status | No rule matches | Rule matches |
|---|---|---|
| Known | `allow` (always) | Rule's action |
| Unknown | `unknown_client` policy | Rule's action |

**With the default `unknown_client = "allow"`:** Every client behaves identically. A client claiming to be `"ollama-local"` that you never configured gets the same treatment as any other client — whatever your rules say. This is the safe default for getting started.

**With `unknown_client = "deny"`:** Any client whose name doesn't appear in a `model_scope` is blocked entirely. Prevents a rogue or misconfigured application from receiving context by using a name you haven't explicitly permitted.

### Example: lock down to known clients only

```toml
[defaults]
unknown_client = "deny"

# Claude Desktop is a known client — health and financial are blocked,
# everything else is allowed (known client allow-fallback applies).
[[rules]]
id = "block-health-claude"
rule_type = "sensitivity"
pattern = "health"
action = "block"
model_scope = "claude*"

[[rules]]
id = "block-financial-claude"
rule_type = "sensitivity"
pattern = "financial"
action = "block"
model_scope = "claude*"

# Ollama is a known client — no restrictions.
[[rules]]
id = "allow-all-ollama"
rule_type = "sensitivity"
pattern = "*"
action = "allow"
model_scope = "ollama*"

# Any other client name → unknown → deny policy → blocked entirely.
```

---

## Audit log

Every injection event is recorded in the DuckDB store under the `injection_log` table:

```
mycontextport inspect --log
```

Each entry records: timestamp, model name, which item IDs were injected, which were blocked, and which rule IDs fired. You can inspect this directly:

```bash
# Using DuckDB CLI
duckdb ~/.local/share/MyContextPort/context.db \
  "SELECT * FROM injection_log ORDER BY injected_at DESC LIMIT 10;"
```
