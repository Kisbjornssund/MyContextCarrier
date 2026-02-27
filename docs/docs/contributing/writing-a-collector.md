---
sidebar_position: 1
title: Writing a Collector
description: Build a context collector for any data source in an afternoon.
---

# Writing a Collector

A collector is a Python class that reads from a local data source and returns `ContextItem` objects. Collectors are the primary contributor pathway. If you use a tool that ContextGenOS does not support yet, you can add it yourself.

**Prerequisites:** Python basics. Access to the data source you want to collect from.

**Estimated time:** Afternoon for a production-quality collector.

---

## Step 1: Scaffold

Generate a complete collector stub:

```bash
contextgenos dev new-collector --name my-tool --platform macos,linux
```

This creates:

```
collectors/my-tool/
├── collector.py          # Your implementation goes here
├── config_schema.py      # Pydantic schema for user config
├── tests/
│   └── test_collector.py # Pre-written test stubs to fill in
└── README.md             # Documentation template
```

---

## Step 2: Implement

Open `collectors/my-tool/collector.py`. Fill in the two required methods:

```python
from contextgenos import BaseCollector, ContextItem, CollectorHealth
from contextgenos.types import Sensitivity

class MyToolCollector(BaseCollector):
    name = "my-tool"
    version = "0.1.0"
    platforms = ["macos", "linux"]

    async def collect(self) -> list[ContextItem]:
        """
        Read context from your data source.
        IMPORTANT: Do not make network requests here.
        """
        items = []

        # Example: read from a local SQLite database
        import sqlite3
        db_path = self.config.get("db_path", "~/.my-tool/data.db")

        # ... your implementation

        return items

    async def health_check(self) -> CollectorHealth:
        """
        Check whether the data source is available on this machine.
        Return healthy=False with a helpful message if it is not.
        """
        import os
        db_path = os.path.expanduser(self.config.get("db_path", "~/.my-tool/data.db"))
        if os.path.exists(db_path):
            return CollectorHealth(healthy=True, message=f"Database found at {db_path}")
        return CollectorHealth(healthy=False, message=f"Database not found at {db_path}")
```

### ContextItem fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | str | Yes | The text content to store and index |
| `source` | str | Yes | Your collector's name |
| `url` | str | No | URL or file path for this item |
| `sensitivity` | Sensitivity | No | Privacy classification (default: UNKNOWN) |
| `metadata` | dict | No | Arbitrary key-value data |

### Sensitivity levels

```python
from contextgenos.types import Sensitivity

Sensitivity.PUBLIC      # Blog posts, public documentation
Sensitivity.WORK        # Work notes, project files, GitHub
Sensitivity.PERSONAL    # Browser history, personal notes
Sensitivity.HEALTH      # Health data
Sensitivity.FINANCIAL   # Financial data
```

---

## Step 3: Test

Run the validation tool before writing your own tests:

```bash
contextgenos dev test-collector --collector ./collectors/my-tool/collector.py
```

This checks: interface completeness, schema compliance, no network calls, health_check behavior.

Then run your tests:

```bash
cd sdk/python
pytest ../collectors/my-tool/tests/ -v
```

Your collector must have at least 5 tests. The scaffold generates stubs for these required cases:

1. `collect()` returns a list
2. `health_check()` returns a `CollectorHealth` with a non-empty message
3. Items have correct `source` field
4. Empty results are handled gracefully
5. `health_check()` returns `healthy=False` when the data source is missing

---

## Step 4: Document

Fill in `collectors/my-tool/README.md`:

- What data source this collects from
- What context it provides (and what it does NOT collect)
- Which platforms are supported
- Configuration options with defaults
- How to find the local data source path on each platform

---

## Step 5: Submit a PR

1. Add your collector to `collectors/REGISTRY.md` (shipped section)
2. Open a pull request
3. Fill in the PR template: the collector checklist section is required
4. The `collector-validation.yml` workflow will run automatically

The collector PR review bar is lower than core PRs. If your collector passes validation and tests, it will generally merge within a few days.

---

## Rules every collector must follow

- Must NOT make network requests inside `collect()` or `health_check()`
- Must NOT require cloud credentials for the v0.1 implementation
- Must handle missing data source gracefully (return empty list, not raise)
- Must include at least 5 unit tests
- Must pass `contextgenos dev test-collector` validation

---

## Need help?

Join `#collectors-dev` on [Discord](https://discord.gg/contextgenos). Post your collector idea and the team will help you find the right data source path and implementation approach.
