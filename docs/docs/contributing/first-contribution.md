---
sidebar_position: 2
title: Your First Contribution
description: Go from zero to merged PR in under 3 hours.
---

# Your First Contribution

This guide gets you from "just arrived" to "PR submitted" in under 3 hours.

---

## Step 1: Set up your environment (30 minutes)

```bash
git clone https://github.com/Kisbjornssund/MyContextPort.git
cd MyContextPort
./scripts/dev-setup.sh
```

The setup script installs: Rust toolchain, Python 3.10+, uv, and Node.js (for the docs site).

Verify everything works:

```bash
cargo build --workspace        # Should complete without errors
cd sdk/python && pytest tests/ # Should pass all tests
```

You are now a MyContextPort developer.

---

## Step 2: Choose your entry point

Pick the option that matches your experience and time available:

### Option A: Fix a documentation issue (30 minutes, any experience)

Look for issues labeled [`area: docs` + `good first issue`](https://github.com/Kisbjornssund/MyContextPort/issues?q=is%3Aopen+label%3A%22good+first+issue%22+label%3A%22area%3A+docs%22).

These are usually: missing steps in a guide, a confusing error message, or an outdated code example. Find one, fix it, open a PR.

### Option B: Add a test (1 hour, any experience)

Look for issues labeled [`good first issue`](https://github.com/Kisbjornssund/MyContextPort/issues?q=is%3Aopen+label%3A%22good+first+issue%22) and search for "add test" or "missing test coverage."

Run the existing tests first to understand the patterns:

```bash
cd sdk/python && pytest tests/ -v
```

### Option C: Complete a collector stub (2 hours, Python)

Look for collector issues with a pre-created stub (the issue will say "stub available").

The stub has all the scaffolding in place. You only need to write the data extraction logic. Read [Writing a Collector](writing-a-collector.md) first.

### Option D: Build a new collector (afternoon, Python)

Pick any data source from the [Collector Registry](https://github.com/Kisbjornssund/MyContextPort/blob/main/collectors/REGISTRY.md) under "Requested." Read [Writing a Collector](writing-a-collector.md) and scaffold it:

```bash
mycontextport dev new-collector --name your-tool
```

---

## Step 3: Make your change

1. Create a branch: `git checkout -b fix/your-description` or `git checkout -b feat/collector-name`
2. Make your changes
3. Run the relevant tests to confirm they pass
4. For Python changes: run `ruff check . && mypy mycontextport/` from `sdk/python/`
5. For Rust changes: run `cargo clippy --workspace && cargo fmt --check`

---

## Step 4: Open a PR

```bash
git push -u origin your-branch
```

Then open a pull request on GitHub. Fill in the PR template completely, especially the Privacy Impact section. Link the issue you are addressing with `Closes #123`.

**What to expect:**
- The `auto-response.yml` workflow will post a welcome message
- CI will run and report results
- A maintainer will give feedback within 3 business days
- For collector PRs, review is usually faster

---

## Getting unstuck

If you hit a wall at any point:

- Post in `#contributing` on [Discord](https://discord.gg/NvqtCBRr)
- Comment on the GitHub issue
- The team is responsive and will help you get unstuck

First contributions are explicitly welcomed. No question is too basic.
