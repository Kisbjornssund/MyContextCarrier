# MyContextPort — Frequently Asked Questions

## The complete Q&A for users, contributors, skeptics, and engineers

---

## Table of Contents

1. [The Basics](#the-basics)
2. [Privacy & Security](#privacy--security)
3. [Technical Architecture](#technical-architecture)
4. [RAG, Memory & AI Concepts](#rag-memory--ai-concepts)
5. [Installation & Setup](#installation--setup)
6. [Data Sources & Collectors](#data-sources--collectors)
7. [AI Model Integration](#ai-model-integration)
8. [For Contributors](#for-contributors)
9. [Business & Sustainability](#business--sustainability)
10. [Hard Questions & Skepticism](#hard-questions--skepticism)
11. [Edge Cases & Failure Modes](#edge-cases--failure-modes)

---

## The Basics

**What is MyContextPort in one sentence?**
/lo
MyContextPort is an open source, locally-run system that automatically learns your personal and professional context from your digital life and makes it available to any AI model you use. Privately, portably, and without sending your data anywhere.

---

**Why does this need to exist? Don't AI tools already have memory?**

Some do, in limited forms. ChatGPT has memory. Claude has Projects. But these are siloed, the memory in ChatGPT doesn't help you in Claude, and neither helps you in Cursor or any other AI tool. More importantly, all of them store your most personal context on servers you don't control, under terms that allow the provider to use your data to improve their models.

MyContextPort is the context layer that works across every AI tool, runs entirely on your machine, and is owned entirely by you. Nothing like this exists in open source today.

---

**What problem does this actually solve day-to-day?**

You open Claude and it doesn't know what you're working on. You open Cursor and it doesn't know your project conventions. You open Perplexity and it doesn't know what you've already researched. Every time you start a new AI conversation, you spend the first few exchanges re-establishing context that you've explained dozens of times before.

With MyContextPort running, every AI conversation you start already knows your active projects, your current focus, your preferences, your history on the topic, and relevant context from your calendar, notes, and recent work without you typing a word of setup.

---

**Who is MyContextPort for?**

Anyone who uses AI tools regularly and wants them to actually know who they are. In practice, the early adopter profile is:

- Developers who use multiple AI coding tools and are tired of re-explaining their stack
- Knowledge workers with complex, ongoing projects spanning multiple tools
- Researchers who want AI to understand their domain and prior work
- Privacy-conscious users who want AI memory without cloud surveillance
- Anyone who has ever pasted the same background context into an AI chat more than three times

---

**Is this a product or a protocol?**

It is infrastructure. MyContextPort is not trying to be a consumer product with a subscription model or a protocol seeking adoption as a standard. It is open source middleware, the layer between your data and your AI tools that anyone can use, modify, extend, or build products on top of.

---

**How is MyContextPort different from just writing a good system prompt?**

A system prompt is static. You write it once, it says the same things every time, and it becomes outdated the moment your situation changes. It also takes up valuable context window space with content that may be irrelevant to the current conversation.

MyContextPort is dynamic. It retrieves and injects only the context that is relevant to your current query, from a living store that is updated continuously as your work and life evolve. The injected context is always current, always targeted, and never wastes context window space on irrelevance.

A system prompt is like a photograph. MyContextPort is a live feed.

---

**How is MyContextPort different from uploading documents to a Claude Project or GPT knowledge base?**

Several important ways. Document uploads are manual, static, and single-model. You have to decide what to upload, upload it yourself, update it manually when things change, and the knowledge base only works in that one application.

MyContextPort is automatic, dynamic, and universal. It collects context continuously from your actual data sources without manual curation. It updates in real time as your situation changes. And the same context store serves every AI tool you use through a universal API: Claude, GPT, local models, whatever you choose.

---

## Privacy & Security

**Where is my data stored?**

On your machine. Specifically, in an encrypted directory at `~/.mycontextport/store/` by default, configurable to any local path. No data is transmitted to MyContextPort servers (there are none) or to any third party as part of normal operation.

---

**What does "local-first" actually mean technically?**

The MyContextPort daemon runs on your machine and makes no outbound network connections during normal operation. You can verify this with any network monitoring tool: Little Snitch on macOS, Wireshark, or a simple `lsof -i` check. The system reads from your local data sources, writes to your local store, and serves context to local applications. The word "local-first" is verifiable, not marketing.

---

**Can MyContextPort see my data?**

MyContextPort contributors and maintainers cannot see your data because there is no server receiving it. The software processes your data locally. If you are asking whether the MyContextPort application itself reads your data: yes, that is how it builds your context store. The application runs on your machine, reads from your sources, and writes to your encrypted local store. No one else reads it.

---

**You say the store is encrypted. Who holds the keys?**

You do. MyContextPort derives encryption keys from your device's secure hardware where available: the Secure Enclave on Apple Silicon, TPM on supported Linux and Windows systems. On hardware without secure enclaves, keys are derived from a user passphrase using Argon2id. The key derivation happens locally and the keys never leave your device. MyContextPort contributors do not have your keys. There is no key escrow. If you lose your keys, your data is unrecoverable — which is the correct tradeoff for a system where the goal is that only you can read your data.

---

**Doesn't injecting my context into Claude or GPT mean my data still reaches their servers?**

Yes, and this is an important distinction. MyContextPort protects your data in storage and controls what gets injected. But when you choose to inject context into a cloud model like Claude or GPT-4, that context leaves your machine as part of the prompt, because the prompt is sent to the model provider's servers.

MyContextPort addresses this through its privacy rules engine, which allows you to configure stricter injection limits for cloud models than for local models. You might permit full personal context to be injected into a locally-running Llama model while restricting cloud models to work-only context. But MyContextPort cannot prevent data from reaching a cloud model if you have configured it to inject that data into cloud model prompts. The transparency is intentional: the injection audit log shows you exactly what was sent to which model.

---

**What happens to my data if I uninstall MyContextPort?**

Your data remains at `~/.mycontextport/` until you delete it. MyContextPort does not perform any cloud cleanup because there is no cloud. Uninstalling the application leaves your local store intact. Run `rm -rf ~/.mycontextport` to delete everything. This is immediate and complete: there is no "we'll delete it within 30 days" because there is no server copy to schedule deletion for.

---

**Can law enforcement access my MyContextPort data?**

A legal demand would have to be directed at you personally, since there is no third-party server holding your data. MyContextPort cannot receive a subpoena or National Security Letter because it is not a company with servers: it is software running on your machine. The legal exposure is the same as for any other encrypted data on your personal device, which is governed by device encryption law in your jurisdiction, not by a cloud provider's legal team navigating a government demand on your behalf without telling you.

---

**Is MyContextPort GDPR compliant?**

GDPR governs the handling of personal data by data controllers and processors: entities that collect and process data about individuals. MyContextPort does not collect your data; it processes your data locally on your own machine, making you the data controller of your own data. This places MyContextPort outside the scope of GDPR obligations that apply to cloud services. The architecture that best serves user privacy is also the architecture that sits most cleanly outside the regulatory framework designed to protect people from companies that collect their data.

---

**How do I know the open source code is actually what's running in the binary I downloaded?**

MyContextPort is committed to reproducible builds. This means the build process is deterministic: anyone who builds the software from the published source code will produce a binary that is cryptographically identical to the binary we distribute. You can verify this by building from source and comparing the hash. This closes the supply chain attack vector where trustworthy source code is compiled into an untrustworthy binary by the distributor.

---

## Technical Architecture

**What is the core technology stack?**

- **Rust**: core daemon for performance, memory safety, and minimal resource footprint
- **DuckDB**: embedded structured data store (no separate database server required)
- **Qdrant**: embedded vector search (runs in-process, no separate service)
- **Knowledge graph**: custom lightweight graph store for entity relationships
- **MCP Protocol**: Model Context Protocol for AI model integration
- **Python SDK**: for plugin development and contributor accessibility
- **React + Tauri**: desktop inspection UI (native app, no Electron overhead)

---

**Why Rust for the core?**

Three reasons. Performance: the daemon runs continuously in the background and needs to be invisible in resource consumption. Memory safety: the daemon handles sensitive personal data and a memory safety bug that leads to data exposure would be a serious privacy failure. Binary size and startup: a system daemon needs to start fast and stay small.

---

**Why DuckDB and not SQLite or PostgreSQL?**

SQLite is excellent but limited for the analytical query patterns MyContextPort uses: time-range queries, aggregations across sources, and complex joins across the context graph. PostgreSQL would require a running server process, which is unnecessarily heavy for a local system. DuckDB is embedded like SQLite but designed for analytical workloads, runs in-process, and handles the query patterns personal context requires naturally.

---

**Why Qdrant embedded and not Chroma, Pinecone, or Weaviate?**

Chroma is excellent for development but not designed for production embedded deployment at the performance level a background daemon requires. Pinecone and Weaviate are cloud services, which violates the local-first constraint immediately. Qdrant supports a true embedded mode with production-grade performance and is the only vector store in the ecosystem that satisfies both the local-first requirement and the performance requirement simultaneously.

---

**How does the context graph work?**

As context is collected from each source, MyContextPort runs lightweight entity extraction to identify people, projects, organizations, topics, and tools mentioned in the content. These entities become nodes in the context graph. Co-occurrence, explicit references, and structural relationships in the source data create edges between nodes. Over time the graph builds a structured representation of your world: who is involved in which projects, which tools are associated with which work streams, which topics cluster together in your thinking.

When you query for context, the graph is traversed alongside the vector index. A query about a project returns not just semantically similar text but the graph neighborhood of that project: the people involved, the related tools, the decisions made, the historical context, structured and ranked by relevance and recency.

---

**How much disk space does MyContextPort use?**

Depends on the number and type of collectors enabled and the retention window configured. A typical deployment with browser, calendar, notes, and GitHub collectors enabled, a 90-day retention window, and standard compression runs between 500MB and 2GB. The embeddings are the largest component. MyContextPort includes configurable retention policies and compression options for users on constrained storage.

---

**How much CPU and RAM does the daemon use?**

In steady state (no active ingestion), the daemon uses less than 1% CPU and approximately 150-300MB RAM. During active ingestion (initial setup or large batch collection), CPU usage will spike but the daemon is designed to yield to foreground processes and will throttle itself to avoid impacting system responsiveness. The system is designed to be invisible during normal operation.

---

**Does MyContextPort work on Windows?**

Yes, with some limitations in the initial release. The core daemon and MCP server are cross-platform. Some collectors (particularly browser history extraction) have platform-specific implementations that are more mature on macOS and Linux in v0.1. Windows TPM support for key derivation is implemented. The desktop UI is cross-platform via Tauri. Full Windows parity is a v0.2 target.

---

**Can MyContextPort run on a home server or NAS so multiple family members or devices can share context?**

Yes, this is a supported deployment model called "household server mode." MyContextPort can be deployed on a local network server where multiple devices connect to a shared context store over your local network. Each user has their own encrypted context partition. Nothing leaves the local network. This is distinct from cloud sync: all data stays within your physical space. Multi-device sync over the internet using user-hosted encrypted storage is a v0.3 feature.

---

## RAG, Memory & AI Concepts

**Isn't this just RAG?**

MyContextPort uses RAG as its vector retrieval mechanism. RAG is a technique. MyContextPort is a system. The difference is everything built around the retrieval: the automatic collection pipeline, the privacy governance layer, the model trust system, the context graph, the encryption architecture, the audit log, the injection transparency, and the universal model API. A detailed technical comparison is in `CONTEXTGENOS_VS_RAG.md`.

---

**How is this different from vector databases like Chroma, Pinecone, or Weaviate?**

Vector databases are storage and retrieval infrastructure. They store embeddings and return nearest neighbors for a query vector. They have no concept of data collection, governance, sensitivity, model trust, audit logging, or user-facing transparency. They are components. MyContextPort is a system that uses an embedded vector store (Qdrant) as one of its components, alongside a structured store, a graph store, a governance layer, a collection pipeline, and a model API.

---

**How is this different from LangChain or LlamaIndex memory modules?**

LangChain and LlamaIndex are developer frameworks for building AI applications. Their memory modules are designed to be used by developers building applications: they require code to configure, they don't run autonomously, and they are not designed to operate as a persistent personal context layer across multiple unrelated applications.

MyContextPort is an autonomous local service: a daemon that runs continuously, collects context automatically, and serves it to any AI application through a universal API, regardless of what framework that application was built with. The relationship is complementary: a developer building an application with LangChain could use MyContextPort as the personal context backend rather than implementing their own memory module.

---

**What's the difference between MyContextPort and the memory feature in ChatGPT or Claude?**

Four key differences. First, locality: MyContextPort runs on your machine, app memory runs on the provider's servers. Second, portability: MyContextPort context works with every AI tool, app memory is siloed to one product. Third, transparency: you can inspect every piece of context MyContextPort holds and every injection it makes; app memory is largely opaque. Fourth, control: MyContextPort gives you fine-grained rules over what is captured and injected; app memory gives you a toggle.

---

**What is MCP and why does it matter?**

MCP (Model Context Protocol) is an open standard developed by Anthropic for how AI models receive external context from tools and services. It is becoming the standard interface for AI model augmentation, similar to how HTTP became the standard interface for web communication. MyContextPort implements a native MCP server, which means any AI tool that supports MCP can receive personal context from MyContextPort without custom integration work. As MCP adoption grows, MyContextPort becomes more useful without any additional development.

---

**Will MyContextPort work with local models like Ollama, LM Studio, or llama.cpp?**

Yes, and local models are actually the highest-trust integration in the MyContextPort model. When you inject context into a local model, your data stays entirely on your machine: the context goes from your local MyContextPort store into a local model running on your hardware, and never touches the internet. MyContextPort supports local models as first-class integration targets via the MCP protocol and direct API integration.

---

## Installation & Setup

**How long does initial setup take?**

The installation itself takes under five minutes. The initial context ingestion, where MyContextPort reads your existing data sources and builds your initial context store, depends on how many collectors you enable and how much data exists. Browser history and notes ingestion for a typical user completes in 15-30 minutes. Email metadata ingestion can take longer depending on mailbox size. You can start using MyContextPort before ingestion is complete — it serves whatever context it has already collected.

---

**Do I need to be technical to use MyContextPort?**

For v0.1, yes, some technical comfort is required. The install and setup process uses a command line interface and configuration files. The v0.2 roadmap includes a graphical onboarding wizard that removes the command line requirement for basic setup. The desktop UI for inspecting and managing your context is graphical and designed for non-technical users.

---

**What permissions does MyContextPort need?**

Permissions depend on which collectors you enable. The daemon itself requires no special permissions to run. Individual collectors require:
- Browser history: read access to browser profile directories (no network permission required)
- Calendar: read access to local calendar files or OAuth token for cloud calendar sync (optional)
- Email: IMAP credentials (metadata-only mode available, which never reads email body)
- GitHub: personal access token with read-only repository scope
- Notes/files: read access to configured directory paths

MyContextPort never requests write access to any data source. It is read-only at the collection layer.

---

**Can I run MyContextPort on a machine without internet access?**

Yes, completely. The core system has no internet dependencies at runtime. Collectors that pull from cloud services (GitHub API, Google Calendar) require internet access for those specific collectors, but these are optional. All local collectors (browser history, local files, local calendar) work fully offline. The MCP server and context injection work fully offline for local models.

---

**How do I migrate MyContextPort to a new machine?**

Export your context store using `mycontextport export --full`, transfer the encrypted archive to your new machine, install MyContextPort, and run `mycontextport import`. Because the store is encrypted with keys derived from your device's secure hardware, you will need to re-derive keys on the new machine using your passphrase backup. The migration guide in `docs/migration.md` covers this in detail.

---

## Data Sources & Collectors

**Which data sources does MyContextPort support?**

**v0.1 (launch):**
- Browser history (Chrome, Firefox, Safari)
- Local calendar files (iCal format)
- Markdown and plain text notes (Obsidian, Logseq, plain directories)
- Google Calendar (via OAuth, optional)

**v0.2:**
- GitHub (repositories, issues, PRs, commit history)
- Email metadata (IMAP, metadata-only mode: sender, subject, timestamps, never body)
- Linear and Jira
- Slack (message metadata and content with user permission)

**v0.3 and community:**
- Notion, Confluence
- Spotify (listening patterns for interest inference)
- Health data (Apple Health, Garmin, Oura, highly restricted by default)
- Financial metadata (transaction categories only, never amounts or accounts by default)
- VS Code activity
- Terminal history

Community-contributed collectors are welcomed and architecturally straightforward to build.

---

**Does MyContextPort read my emails?**

By default, no. The email collector operates in metadata-only mode, which reads sender, recipient, subject, and timestamps, never the email body. This provides significant context signal (who you communicate with, about what topics at a high level, communication patterns) without exposing the content of private correspondence.

A full-content email mode is available for users who want richer context from their email, with explicit opt-in and additional sensitivity classification applied to all email-derived context. Full-content email context is treated as high-sensitivity and is restricted from injection into cloud models by default.

---

**What does MyContextPort do with my browser history?**

It reads the URLs and page titles from your local browser history database (stored on your machine by your browser) and extracts topic and interest signals from them. It does not read page content. It does not intercept live browsing traffic. Domain-based exclusion rules allow you to prevent any domain or domain pattern from being processed, for example `mycontextport collector config browser --exclude "*.bank.com"`.

---

**Can I exclude specific data from being captured?**

Yes, at multiple levels. You can exclude entire domains, file paths, email senders, calendar categories, or time ranges from collection. You can delete specific context items after they've been collected. You can mark entire sensitivity categories as never-capture. The privacy rules engine documentation covers the full configuration syntax.

---

**Does MyContextPort capture what I type or what I do on screen?**

No. MyContextPort is not a keylogger or screen recorder. It reads from structured data sources: database files your browser creates, calendar files your calendar app creates, note files you've written, API responses from services you've authorized. It does not monitor keystrokes, capture screenshots, or observe your screen. The collection is explicitly pull-based from known data sources, not observation-based from system activity.

---

**I'm worried about a specific piece of data being captured. How do I check?**

Run `mycontextport inspect --source browser --recent` (replacing browser with any source) to see the most recent items collected from that source. Run `mycontextport inspect --search "keyword"` to search your context store for specific content. Run `mycontextport delete --search "keyword"` to delete matching items. The full inspection UI provides a browsable view of everything in your store.

---

## AI Model Integration

**Which AI models does MyContextPort work with?**

Any model that supports MCP (Model Context Protocol), which includes Claude, and any model accessible via standard API with context injection support. For models that don't yet natively support MCP, MyContextPort provides a proxy integration mode where it intercepts prompts, injects context, and forwards to the model API. Local models via Ollama, LM Studio, and llama.cpp are supported natively.

---

**How do I connect MyContextPort to Claude?**

Add the MyContextPort MCP server to your Claude configuration:

```json
{
  "mcp_servers": [
    {
      "name": "mycontextport",
      "url": "http://localhost:8765/mcp"
    }
  ]
}
```

Claude will automatically request relevant context from MyContextPort for each conversation. No further configuration is required.

---

**How do I connect MyContextPort to a local model running in Ollama?**

```bash
mycontextport integration add ollama --model llama3
```

MyContextPort will automatically inject context into prompts sent to the configured Ollama model via its local API. Local model integrations default to the highest trust level, allowing full personal context injection.

---

**Will context injection make my AI responses slower?**

The latency impact is minimal in practice. Context retrieval from the local store is fast, typically under 50ms for a retrieval query, and happens in parallel with the prompt being formed. For cloud models where network latency dominates, the MyContextPort retrieval adds negligible time. For local models, the retrieval and the model inference can be pipelined to eliminate the overhead entirely.

---

**How much of my context window does MyContextPort use?**

MyContextPort is context-window aware. It receives the maximum context budget available for the model being used and selects the highest-relevance context items that fit within a configurable fraction of that budget (default: 20% of available context window). This ensures that context injection never crowds out the actual conversation. The budget fraction is user-configurable per model.

---

**What if the injected context is wrong or outdated?**

You can correct it. The inspection UI allows you to edit, delete, or update any context item in your store. You can also give real-time corrections in conversation: if MyContextPort injects something outdated, you can tell the AI "that's outdated, actually X" and MyContextPort will update its store based on your correction (with your permission). Feedback loops from conversations back into the context store are a v0.2 feature.

---

## For Contributors

**I want to contribute. Where do I start?**

Read `CONTRIBUTING.md` for the full guide. The highest-leverage entry points for new contributors are:

- **Context collectors:** pick a data source you use daily and build a collector plugin. The collector spec is in `docs/collector-spec.md` and a reference implementation is in `collectors/browser/`. A basic collector is achievable in an afternoon with Python familiarity.
- **Model integrations:** if you use an AI tool that doesn't yet have a MyContextPort integration, building one is straightforward and immediately valuable to other users of that tool.
- **Documentation:** clear technical writing is among the most valuable contributions an open source project receives. Unclear docs prevent adoption more than missing features.
- **Testing:** the test coverage in early releases will be incomplete. Adding test coverage is always welcome and helps you understand the codebase deeply.

---

**What does a context collector look like technically?**

A collector is a Python class implementing the `BaseCollector` interface:

```python
class MyAppCollector(BaseCollector):
    name = "myapp"
    description = "Collects context from MyApp"
    sensitivity_default = SensitivityLevel.WORK

    def authenticate(self) -> bool:
        # Handle any auth required to access the data source
        pass

    def collect(self, since: datetime) -> Iterator[ContextItem]:
        # Yield ContextItem objects from the data source
        # since: only collect items newer than this timestamp
        pass

    def health_check(self) -> CollectorHealth:
        # Return the current health status of this collector
        pass
```

MyContextPort handles storage, embedding, encryption, and injection. The collector only needs to handle extraction from the source and yield structured items.

---

**Can I contribute a collector for a service with a cloud API?**

Yes, with one guideline: cloud API collectors must clearly document that they make outbound network requests and what data they transmit. Users must explicitly opt in to cloud API collectors during setup. The collection still happens locally (your machine calls the API, processes the response, stores results locally): the distinction is that the collector touches the internet, unlike purely local collectors.

---

**Is there a bounty or paid contribution program?**

Not currently. MyContextPort is a pure open source project with no commercial entity behind it at launch. If the project generates revenue through foundation grants, sponsorships, or related commercial products, contributor compensation will be discussed publicly. We will never introduce a paid-contributor program that creates unequal influence over the project's direction.

---

**Can I build a commercial product on top of MyContextPort?**

Yes. The MIT license permits commercial use, including building proprietary products on top of MyContextPort. You do not need to open source your product. You do not need to pay a license fee. The one requirement is to include the original copyright notice and license text in your software. If you build something significant on MyContextPort, we'd love to know about it, but that's encouragement, not a license condition.

---

**I work at an AI company. Can we integrate MyContextPort into our product?**

Yes. We actively want AI tool providers to integrate MyContextPort as a context source. The MCP server makes this straightforward. We would particularly welcome official MCP support in AI tools that makes MyContextPort configuration a first-class option in their settings, and we will work with AI tool teams to make integration as smooth as possible.

---

## Business & Sustainability

**Who is behind MyContextPort?**

MyContextPort was initiated as a personal open source project by independent contributors. There is no company, no VC funding, and no commercial entity operating the project at launch. The project is governed as a standard open source project: decisions are made through public discussion, code review, and contributor consensus.

---

**How is MyContextPort funded?**

Initially through contributor time. The long-term sustainability model being explored includes: open source foundation grants (NLNet, Mozilla Foundation, Sovereign Tech Fund), optional cloud features for users who want hosted sync under their own control, and commercial support contracts for enterprise deployments. No funding model that compromises the local-first, privacy-first architecture will be accepted.

---

**Is there a company that could get acquired and change the terms?**

There is no company. The MIT license cannot be revoked for code already published under it, even if a company were formed and acquired, the already-published open source code remains open source under MIT forever. This is one of the most permissive licenses available, chosen so the software remains free without restriction.

---

**What happens if the main contributors stop working on it?**

MyContextPort is designed to be forkable and self-sustaining. The architecture documentation is comprehensive. The plugin system means the project can remain useful even without active core development: collectors and integrations can be maintained independently by their contributors. If the main contributors move on, the project continues under new maintainers or as a maintained fork. This is the natural lifecycle of healthy open source infrastructure.

---

**Will MyContextPort stay free?**

The core system will always be free and open source under MIT. Optional premium features, such as hosted encrypted cloud sync for users who want cross-device context without running their own server, may be offered commercially in future. Any commercial features will be additive, never gating functionality that was previously free.

---

## Hard Questions & Skepticism

**This sounds like it would make AI systems know terrifyingly much about me. Isn't that concerning?**

It is a reasonable concern and worth sitting with. The honest answer is: yes, a system that aggregates your browser history, calendar, notes, emails, and GitHub activity into a unified context store knows a lot about you. This is powerful and it is personal.

The counterargument is not that this is fine and you shouldn't worry. The counterargument is that this aggregated context is being built and used by cloud AI services right now, except that it sits on their servers, under their terms, used in ways you can't audit, with deletion you can't verify. MyContextPort does not make the aggregation safer by making it not exist. It makes it safer by putting you in control of it. The choice is not between "AI that knows you" and "AI that doesn't." It is between "AI that knows you, owned by a corporation" and "AI that knows you, owned by you."

If you are genuinely uncomfortable with any system aggregating this much personal context, that is a legitimate position, and MyContextPort's granular capture rules allow you to keep it as minimal as you are comfortable with.

---

**What stops MyContextPort from being used to build surveillance tools?**

The architecture. MyContextPort is a pull-based system: it reads from existing local data stores that your applications already create. It does not install keystroke loggers, screen recorders, or network interceptors. A surveillance tool built on MyContextPort would have the same capabilities as a surveillance tool built directly on the underlying data sources, which is to say, MyContextPort adds nothing to the surveillance surface. The privacy rules engine, transparency tooling, and inspection UI are all explicitly designed to give the person on the monitored machine visibility and control, which is the opposite of what surveillance tools want.

We will not build features that enable covert deployment. MyContextPort will always be visible and controllable by the person whose machine it runs on.

---

**AI companies will just build this into their products and make MyContextPort irrelevant.**

Maybe. This outcome would be broadly good for users if the in-product version is local, portable, and transparent. We would celebrate AI tools competing on privacy and context quality. If this happens, MyContextPort will have succeeded in its mission by making the category exist and setting the standard that proprietary implementations have to meet.

More likely, AI companies will build context features that are deeply integrated with their own products, siloed from competitors, cloud-dependent, and monetized through data accumulation. MyContextPort exists for the users who do not accept that bargain.

---

**LLM context windows are getting very large. Won't context just become irrelevant when models can hold everything?**

Long context windows are genuinely impressive and change the tradeoffs meaningfully. But several things remain true even at 1 million token context windows:

Cost per token means filling a massive context window is expensive in both money and latency. Selection still matters: injecting everything you've ever done is less useful than injecting what's relevant now. Privacy still matters: a large context window on a cloud model means more of your personal data sent to a cloud server per query. The collection problem remains: someone has to gather, structure, and maintain your context whether the window is large or small.

MyContextPort becomes more valuable as context windows grow because it becomes the system that intelligently selects what to put into the large window available, rather than blindly filling it.

---

**What if the AI models MyContextPort injects into use the injected context to train their models?**

For cloud models, this is a real concern and is governed by the model provider's terms of service, not by MyContextPort. MyContextPort gives you control over what is injected and can restrict sensitive context from cloud model injection. For local models, training on injected context is not possible since local inference does not transmit data anywhere.

The best protection against training data concerns for cloud models is to configure MyContextPort to inject only work-level, non-sensitive context into cloud models, and to use local models for conversations where you want richer personal context. MyContextPort's per-model injection rules make this configuration straightforward.

---

**Is the context graph actually useful or is it architecture astronautics?**

Fair skepticism. The context graph is a genuine differentiator in specific scenarios: multi-entity queries, relationship traversal, temporal reasoning about how your situation has evolved. For simple "what am I working on" queries, vector retrieval alone is sufficient and the graph adds little.

The graph earns its complexity in scenarios like: "What was the status of Project X and who was involved when I last touched it six months ago?" or "I'm about to email someone I haven't talked to in a year, what was the history of our collaboration?" These queries require relational and temporal structure that a flat vector index cannot provide. If you never need these queries, the graph is transparent overhead. If you do need them, it is the feature that makes MyContextPort qualitatively more useful than a RAG pipeline.

---

**Why would contributors work on this when they could build a startup doing the same thing?**

Some of them will build startups. Some of those startups will use MyContextPort as their foundation, which is explicitly permitted by the license. The contributors who work on MyContextPort as open source are doing so because they believe that personal context infrastructure should be a public good, the same reason people contribute to Linux, PostgreSQL, Firefox, and Home Assistant instead of building proprietary alternatives to those. The financial incentive argument proves too much: by that logic, no open source infrastructure should exist, and yet it powers most of the internet.

---

**What is the biggest risk to this project succeeding?**

Honest answer: distribution. The technical risks are manageable: the architecture is sound, the component technologies are proven, and the problem is clearly defined. The risk is that MyContextPort remains a tool for engineers and never reaches the broader population of AI users who would benefit from it most. The v0.2 onboarding improvements, the desktop UI, and the integrations into mainstream AI tools are the most important work for reaching beyond early adopters. If MyContextPort remains a CLI tool that requires YAML configuration, it will be loved by engineers and unknown to everyone else.

---

## Edge Cases & Failure Modes

**What happens if the MyContextPort daemon crashes?**

AI tools that request context receive an empty context response and continue functioning normally without context augmentation. The daemon is designed to fail gracefully: a crash should not impact any other application. The daemon auto-restarts on failure via the system service manager (launchd on macOS, systemd on Linux, Windows Service).

---

**What if the context store becomes corrupted?**

The structured store (DuckDB) has WAL (write-ahead log) journaling enabled, providing crash recovery. The vector index (Qdrant) has similar durability guarantees. In the event of corruption, MyContextPort includes a repair command that attempts recovery from the WAL. If recovery fails, the store can be rebuilt from source data by re-running collection, losing only context items that were derived from sources that no longer exist.

---

**What if a context collector starts producing bad data?**

Each collector runs in an isolated process with resource limits. A misbehaving collector cannot corrupt the core store: it can only produce items that fail schema validation and are rejected at ingest. Collector health is monitored and a collector that produces more than a configurable error rate is automatically paused and flagged in the UI.

---

**Can MyContextPort be manipulated through my data sources, e.g., a malicious website injecting context?**

This is a real attack vector — prompt injection through context — and it is taken seriously. MyContextPort applies sanitization to all collected content before storage. Content that contains patterns associated with prompt injection attempts (imperative instructions, role override syntax, delimiter injection) is flagged, quarantined, and not injected without explicit user review. The inspection UI allows you to review and release quarantined items if they are legitimate. This does not provide perfect protection against sophisticated prompt injection through context, but it raises the bar significantly above naive injection.

---

**What if I share my machine with other people?**

MyContextPort context stores are per-user at the OS level. Each user account on a machine has an independent, encrypted context store accessible only when that user is logged in. Shared machine use does not create context cross-contamination between users.

---

**Does MyContextPort work if I use multiple machines?**

In v0.1, each machine has an independent context store: there is no sync between machines. Context collected on your laptop is not available when you use your desktop. Cross-device sync using user-hosted encrypted storage is a v0.3 feature. Household server mode (a local network server accessible by multiple devices) is available earlier and provides cross-device context for users willing to set up a local server.

---

*MyContextPort — github.com/mycontextport/mycontextport*
*Licensed under MIT*
*Have a question not answered here? Open a GitHub Discussion.*
