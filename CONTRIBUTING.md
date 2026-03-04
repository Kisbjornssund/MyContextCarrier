# Contributing to MyContextPort

MyContextPort is built by the people who use it. The plugin architecture is designed so that a meaningful contribution is possible in an afternoon. Every data source you can imagine is a collector waiting to be built.

---

## Where to Start

### Choose your entry point

| Domain | What you build | Prerequisites | Estimated effort |
|--------|---------------|--------------|-----------------|
| [Context Collectors](#context-collectors) | Data source plugins | Python basics | Afternoon |
| [AI Integrations](#ai-integrations) | Connect a new AI tool | Target tool's API | Half-day |
| [Core Daemon](#core-daemon) | Rust performance/features | Rust, async systems | Days |
| [Documentation](#documentation) | Guides, references | Writing ability | 1–3 hours |

**Not sure where to start?** Check [open issues labeled `good first issue`](https://github.com/Kisbjornssund/MyContextPort/issues?q=is%3Aopen+label%3A%22good+first+issue%22).

**Have a question?** Join `#contributing` on [Discord](https://discord.gg/NvqtCBRr) before opening an issue.

---

## Contribution Types and Process

### Bugs and small fixes
Open a PR directly. No prior discussion needed. Include reproduction steps and your environment.

### New collectors and AI integrations
These are the most welcome contributions. For new collectors, no prior discussion is needed, just follow the spec below and open a PR.

### New core features or architecture changes
Start a [GitHub Discussion](https://github.com/Kisbjornssund/MyContextPort/discussions) or ask in `#core-dev` on Discord before writing code. This prevents wasted effort on approaches that won't be merged.

### Questions
Use Discord `#getting-started` or `#collectors` before opening a GitHub issue.

---

## Context Collectors

Collectors are the primary contributor pathway. Each collector is a Python class that reads from a data source on the user's machine and returns structured `ContextItem` objects.

### Quick start

Generate a collector scaffold:
```bash
mycontextport dev new-collector --name my-tool --platform macos,linux
```

This creates:
- `collectors/my-tool/collector.py`: The Python class with all interface methods stubbed
- `collectors/my-tool/tests/test_collector.py`: Test file with required test cases
- `collectors/my-tool/README.md`: Documentation template
- `collectors/my-tool/config_schema.py`: Pydantic schema for collector config

Implement the data extraction logic, run the tests, submit a PR.

### The BaseCollector interface

```python
from mycontextport import BaseCollector, ContextItem, CollectorHealth
from typing import AsyncIterator

class MyToolCollector(BaseCollector):
    """
    Collect context from My Tool.

    Reads from the local My Tool database at ~/.my-tool/data.db.
    Does not make network requests.
    """

    name = "my-tool"
    version = "0.1.0"
    platforms = ["macos", "linux"]

    async def collect(self) -> list[ContextItem]:
        """Return context items collected from this source."""
        items = []
        # Read from local file, database, or API
        # Do NOT make network requests during collect()
        return items

    async def health_check(self) -> CollectorHealth:
        """Check if this collector can run on the current system."""
        return CollectorHealth(
            healthy=True,
            message="My Tool database found at expected path"
        )

    async def stream(self) -> AsyncIterator[ContextItem]:
        """Optional: stream items as they are collected (for large sources)."""
        for item in await self.collect():
            yield item
```

### Collector requirements

- Must implement `collect()` and `health_check()` completely
- Must not make network requests during `collect()` or tests
- Must include at least 5 unit tests
- Must be tested on the platforms declared in `platforms`
- Must include a `README.md` entry in the collector directory
- Must pass `mycontextport dev test-collector --collector ./collector.py`

### Validate before submitting

```bash
mycontextport dev test-collector --collector ./collectors/my-tool/collector.py
```

This runs: interface completeness check, schema validation, network call detection, health_check verification.

---

## AI Integrations

Integrations connect MyContextPort's MCP server output to specific AI tools.

See `docs/docs/contributing/ai-integrations.md` for the integration spec.

Examples that need building: JetBrains plugin, VS Code extension (beyond MCP), browser extension, Raycast plugin.

---

## Core Daemon

The Rust core handles storage, indexing, the privacy rules engine, and the MCP server.

**Prerequisite:** Comfort with Rust and async programming (tokio).

**Setup:**
```bash
./scripts/dev-setup.sh
cargo build --workspace
cargo test --workspace
```

**Before submitting a core PR:**
```bash
cargo clippy -- -D warnings
cargo fmt --check
cargo audit
```

Core changes require maintainer review. Open a GitHub Discussion before starting work on anything architectural.

---

## Documentation

Documentation contributions are high-leverage and require no code.

Good documentation issues to look for: missing steps in the quickstart, confusing error messages without explanation, collector spec gaps, untested instructions.

To run the docs site locally:
```bash
cd docs
npm install
npm start
```

---

## Development Environment

```bash
# One-command setup
./scripts/dev-setup.sh

# This installs: Rust toolchain, Python 3.10+, uv, Node.js (for docs)
# and runs: cargo build --workspace
```

Requirements:
- macOS 13+ or Linux (Ubuntu 22.04+, Fedora 38+)
- 8 GB RAM minimum
- 4 GB disk space for dependencies

---

## Pull Request Process

1. Fork the repository and create a branch: `git checkout -b feat/my-feature` or `git checkout -b fix/issue-123`
2. Make your changes and write or update tests
3. Run `./scripts/check.sh` (runs cargo test, clippy, fmt, and Python tests)
4. Fill out the pull request template completely: the Privacy Impact and Security Impact sections are required
5. Link any related issues with `Closes #123`

**Review timeline:** The maintainer team aims to give first feedback within 3 business days. Collector PRs typically merge faster than core PRs.

---

## AI-Assisted Contributions

AI-assisted code is welcome. Please disclose in your PR:
- That you used AI assistance
- Your testing level: untested / lightly tested / fully tested with passing tests
- That you understand and can explain the code you are submitting

Undisclosed AI-generated code that introduces bugs or security issues may result in the PR being closed without merge.

---

## Becoming a Maintainer

Maintainers are added deliberately. If you have been an active contributor and want to take on more responsibility:

Email `contributing@mycontextport.dev` with:
- Your GitHub and Discord handles
- A description of your open source background
- Which area of MyContextPort you want to maintain (collectors, core, SDK, docs)
- A realistic estimate of your weekly time commitment

The team reviews applications carefully and responds within 2 weeks.

---

## Security Vulnerabilities

Do not open public GitHub issues for security vulnerabilities. See [SECURITY.md](SECURITY.md) for the responsible disclosure process.

---

## Code of Conduct

By contributing, you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

---

*Questions? Join `#contributing` on [Discord](https://discord.gg/NvqtCBRr).*
