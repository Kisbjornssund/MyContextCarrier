---
sidebar_position: 1
---

# Privacy Rules

MyContextPort evaluates a privacy rules file before injecting any context into an AI model. Rules are evaluated in order — the first matching rule wins. If no rule matches, the item is **allowed** by default.

## Config file location

Place your rules at:

```
~/.local/share/MyContextPort/privacy.toml   # Linux
~/Library/Application Support/MyContextPort/privacy.toml  # macOS
```

The file is created (empty = allow-all) the first time you run `mycontextport init`.

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
