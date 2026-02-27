## Summary

<!-- 2-5 bullets covering: what problem this solves, why it matters, what changed, and what is explicitly NOT in scope -->

-
-
-

## Change type

- [ ] Bug fix
- [ ] New collector
- [ ] AI model integration
- [ ] Core / daemon change
- [ ] Privacy or security change
- [ ] Documentation
- [ ] CI / tooling
- [ ] Refactor
- [ ] Other

## Scope

- [ ] core
- [ ] mcp
- [ ] privacy-engine
- [ ] store
- [ ] graph
- [ ] sdk
- [ ] collector
- [ ] cli
- [ ] ui
- [ ] docs
- [ ] ci

## Linked issues

Closes #

## Privacy impact

<!-- Required. Does this PR change what data is collected, stored, or injected? -->
<!-- If no privacy impact: write "None: this change does not affect data collection or injection." -->

## Impact on user data and security

<!-- Required. Answer all that apply: -->
<!-- - Does this make new network requests? -->
<!-- - Does this access new file paths or system APIs? -->
<!-- - Does this change what is stored in the local context store? -->
<!-- - Does this change what context is injected into AI models? -->
<!-- - Does this introduce new dependencies? -->

## For collector PRs: checklist

- [ ] Implements all `BaseCollector` interface methods
- [ ] Does not make network requests during `collect()` or in tests
- [ ] Includes at least 5 unit tests covering the main logic paths
- [ ] Passes `contextgenos dev test-collector` validation
- [ ] Tested locally on: (check all that apply) macOS / Linux / Windows
- [ ] `collectors/REGISTRY.md` entry added or updated
- [ ] Collector `README.md` written in the collector directory

## Evidence

<!-- Tests, logs, screenshots, or terminal output showing this works -->

## Rollback plan

<!-- If this breaks something after merge, how would we revert or fix it quickly? -->

## AI-assisted code disclosure

<!-- If you used AI assistance (Claude, GPT, Copilot, etc.) to write any part of this PR: -->
- [ ] I used AI assistance
- Testing level: untested / lightly tested / fully tested with passing tests
- I understand the code I am submitting and can explain it if asked
