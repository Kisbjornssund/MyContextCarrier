# Collector Registry

This file tracks all MyContextPort context collectors: shipped, in progress, and requested.

Before building a new collector, check here to avoid duplicate effort.
To add an entry, open a [Collector Request issue](https://github.com/Kisbjornssund/MyContextPort/issues/new?template=collector_request.yml).

---

## Shipped (in this repository)

| Collector | Data source | Platforms | Ships in |
|-----------|------------|-----------|----------|
| [browser](browser/) | Chrome, Firefox browser history | macOS, Linux | v0.1 |
| [notes](notes/) | Markdown files, Obsidian vaults | All | v0.1 |
| [calendar](calendar/) | Google Calendar, iCal | All | v0.2 |

---

## In Progress

| Collector | Data source | Contributor | PR |
|-----------|------------|-------------|-----|
| — | — | — | — |

---

## Requested (not yet built, good first issues)

| Collector | Data source | Issue | Difficulty |
|-----------|------------|-------|------------|
| github | GitHub repos, PRs, issues, commits | [#TBD]() | Medium |
| vscode | VS Code recent files, workspace history | [#TBD]() | Easy |
| email | Email (IMAP, metadata only) | [#TBD]() | Medium |
| slack | Slack message history | [#TBD]() | Medium |
| linear | Linear issues and projects | [#TBD]() | Easy |
| jira | Jira issues | [#TBD]() | Easy |
| notion | Notion pages and databases | [#TBD]() | Medium |
| todoist | Todoist tasks | [#TBD]() | Easy |
| obsidian-advanced | Obsidian graph, tags, backlinks | [#TBD]() | Easy |
| apple-health | Apple Health data | [#TBD]() | Medium |
| spotify | Listening history | [#TBD]() | Easy |
| garmin | Activity and health data | [#TBD]() | Medium |
| zotero | Research library and annotations | [#TBD]() | Easy |

---

## Want to build one?

1. Check the list above for something you use daily
2. Read [Writing a Collector](https://docs.mycontextport.dev/collectors/writing-a-collector)
3. Run `mycontextport dev new-collector --name your-tool` to scaffold
4. Open a PR: collector PRs are the fastest path to a merged contribution

Questions? Join `#collectors-dev` on [Discord](https://discord.gg/NvqtCBRr).
