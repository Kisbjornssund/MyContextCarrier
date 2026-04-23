# MyContextCarrier Privacy & Security Architecture

## *Why Local-First Is Not a Feature: It's a Fundamentally Different Model*

---

## The Problem With Every Existing Solution

Before explaining what MyContextCarrier does, it's worth being precise about what currently exists and exactly where each solution fails, because the failures are not minor inconveniences. They are structural, and they are by design.

### Cloud Memory Services (Mem.ai, Notion AI, etc.)

When you use a cloud-based AI memory service, the following things are true whether or not they are disclosed clearly:

**Your data leaves your machine.** Every note, every captured thought, every piece of context you feed the system travels over the internet to servers you don't control. The encryption in transit (TLS) that these services advertise protects your data from a third-party intercepting the connection, not from the service itself.

**The service holds the encryption keys.** "Encrypted at rest" is the most commonly misunderstood data security claim in consumer software. It means your data is encrypted on their servers, but they hold the decryption keys. Their employees, their systems, their legal counsel, and any government agency that serves them a valid legal demand all have access to your data. You do not have meaningful encryption if the entity you're trusting to keep the data safe also holds the key.

**Your context trains their models.** This is rarely stated plainly. The terms of service of most AI memory and context services include provisions allowing the company to use your data to improve their models. Your professional strategies, your intellectual patterns, your relationship notes, your project history — these become training signal. You are paying for a service while simultaneously providing the labor that makes their core AI asset more valuable. This is not incidental. It is the business model.

**You are locked in.** The context layer is the stickiest possible lock-in mechanism because context compounds over time. The longer you use a cloud memory service, the more intelligence it accumulates about you, and the more painful it is to leave. Your accumulated context becomes a hostage. Some services offer export, but the export format is rarely useful outside their ecosystem, and no competing service can import your context and immediately replicate the intelligence you built up.

**Legal jurisdiction applies.** Your data stored on cloud servers is subject to the laws of the jurisdiction where those servers reside. In the United States, this includes the Electronic Communications Privacy Act, National Security Letters, and FISA court orders — some of which come with gag orders that legally prevent the service from telling you your data was accessed. If your data contains business strategy, intellectual property, or sensitive personal information, this is a material risk.

### App-Specific Memory (ChatGPT Memory, Claude Projects, Gemini)

These systems solve a narrower problem, remembering context within a single product, and they fail in a different but equally significant way.

**Siloed intelligence.** The context ChatGPT builds about you does not help you when you open Claude. The projects you've built in Claude don't inform your Cursor sessions. The memory is owned by the product, not by you. You don't have one intelligent context layer that knows you: you have five partial, disconnected mirrors of yourself, each locked inside a different corporation's product.

**The same cloud problems apply.** Despite the narrower scope, all of the same issues, cloud storage, provider-held keys, training data concerns, legal jurisdiction, apply equally to app-specific memory. The confinement of the memory to one app does not make it more private. It makes it less useful while being equally exposed.

**Memory as a moat.** App-specific memory is explicitly designed to increase switching costs. The more you invest in a particular app's memory system, the harder it is to move to a competitor. This is a feature from the product's perspective. It is a trap from the user's.

### Local Note Systems with AI Plugins (Obsidian, Logseq + AI)

This category is closer to the right idea and deserves more credit, but it falls short in specific, addressable ways.

**Manual curation burden.** These systems require you to manually write, tag, organize, and maintain the context that AI can learn from. The burden of curation falls entirely on the user. This is precisely backwards. The value of an AI context layer is that it learns automatically from your behavior and work, not that it gives you a more sophisticated way to take notes.

**No cross-application context.** Even the best Obsidian AI plugin only knows what's in your vault. It doesn't know about your GitHub activity, your email patterns, your calendar, your browser research, your Slack discussions. Your actual context is distributed across dozens of tools. A notes-based system captures one sliver of it.

**No universal injection.** These plugins inject context into specific AI integrations within their app. They do not provide a universal context API that works with any AI tool you choose to use. The context stays inside the note-taking app ecosystem.

**Plugin fragmentation.** The plugin marketplace for these tools has dozens of partially-maintained AI integrations, each with different capabilities, different privacy characteristics, and different maintenance status. There is no coherent system: there is a collection of experiments.

---

## The MyContextCarrier Model: Architecture as Privacy

MyContextCarrier does not treat privacy as a settings menu or a compliance checkbox. Privacy in MyContextCarrier is an architectural property, meaning it is enforced by how the system is built, not by how the system is configured.

This distinction matters enormously. A system that protects your data through settings can have those settings changed, by you accidentally, by a software update, by a policy change, by a new owner after an acquisition. A system that protects your data through architecture cannot be misconfigured into exposing it, because exposure requires deliberate, visible action.

### The Local-First Guarantee

MyContextCarrier runs entirely on your machine. There is no MyContextCarrier server. There is no MyContextCarrier cloud. There is no company database that aggregates user context. The software you install is the entirety of the system, and it operates exclusively on your hardware.

What this means in practice:

**No network requests at runtime.** The MyContextCarrier daemon, once installed, does not make outbound network connections during normal operation. It reads from your local data sources, writes to your local store, and serves context to local applications. You can verify this with any network monitoring tool. We encourage you to do so.

**No telemetry.** MyContextCarrier collects no usage data, no error reports, no "anonymous analytics." We know that most software says this. The difference here is that you can verify it — the codebase is open, the binary is reproducible, and the network behavior is auditable. Trust should be verifiable, not demanded.

**No authentication to a remote service.** You do not create a MyContextCarrier account. You do not log in. There is no server that could be breached and expose your credentials or your data, because there is no server.

**Your data has one location.** Your context store lives at a path on your filesystem that you can inspect, back up, encrypt with your own tools, or delete entirely. At any moment, you can `rm -rf ~/.mycontextport` and every piece of data MyContextCarrier holds is gone. No deletion requests. No "we'll delete it within 30 days." Gone, immediately, because it was always yours.

### Encryption Architecture

MyContextCarrier implements encryption in a fundamentally different way than cloud services.

**You hold the keys.** The local context store is encrypted using keys derived from your device's secure enclave (on supported hardware: Apple Silicon, TPM on Windows/Linux) or a user-provided passphrase. MyContextCarrier never transmits your keys. MyContextCarrier never generates keys on a remote server. The mathematical relationship between your keys and your data means that no one who does not possess your keys — including the authors of MyContextCarrier — can read your context store.

**Key derivation.** On first initialization, MyContextCarrier derives an encryption key using your device's secure hardware where available. On Apple Silicon, this uses the Secure Enclave. On Linux with TPM, this uses the TPM chip. On systems without secure hardware, a passphrase-derived key (Argon2id) is used. The derived key never leaves your device.

**Encrypted at rest, controlled by you.** The DuckDB structured store and the Qdrant vector index are both encrypted at rest. The encryption is performed by MyContextCarrier before data is written to disk. Unlike cloud "encrypted at rest," there is no server-side decryption layer between your data and any third party.

**Optional additional encryption.** Users who require higher assurance can configure MyContextCarrier to encrypt the entire data directory with a separate passphrase required at daemon startup. This means the context store is unusable even if someone gains physical access to your machine while it is powered off.

### The Privacy Rules Engine

Not all context is equally sensitive, and not all context is appropriate to share with every AI tool. MyContextCarrier includes a privacy rules engine that gives you fine-grained control over what context is captured and what context is injected.

**Capture rules:** Control what MyContextCarrier collects from each source:
```yaml
collectors:
  browser:
    exclude_domains:
      - "*.bank.com"
      - "health*.com"
      - "*.medical"
    exclude_patterns:
      - "password"
      - "social security"
    time_range_exclude:
      - after: "22:00"
        before: "07:00"  # No capture during personal hours
  email:
    mode: "metadata_only"  # Captures sender, subject, timing, never body
    exclude_senders:
      - "therapy@*"
      - "doctor@*"
```

**Injection rules:** Control what context is shared with which AI:
```yaml
injection:
  default_sensitivity: "work"  # Only work-tagged context by default
  per_model:
    claude:
      sensitivity_levels: ["work", "technical"]
    local_llm:
      sensitivity_levels: ["work", "technical", "personal"]
    gpt-4:
      sensitivity_levels: ["work"]  # More restrictive for cloud models
  never_inject:
    - financial_data
    - health_data
    - relationship_notes
```

**Context tagging.** Every piece of context captured by MyContextCarrier is automatically tagged with a sensitivity classification (work, personal, health, financial, relationship) based on its source and content. These tags drive injection decisions. You can review and override tags at any time through the inspection UI.

### The Inspection Guarantee

MyContextCarrier maintains a complete, human-readable log of every context injection event. Before any piece of your context is provided to any AI tool, a log entry is created containing:

- Timestamp
- Which AI tool requested context
- What query or prompt triggered the context retrieval
- Which context items were retrieved
- Which context items were actually injected (after privacy rules were applied)
- Which context items were blocked by privacy rules

This log is accessible through the MyContextCarrier UI and queryable via CLI:

```bash
# What did MyContextCarrier inject into my Claude conversation this morning?
mycontextport log inspect --since "09:00" --model claude

# Has anything from my health data ever been injected?
mycontextport log search --sensitivity health

# What does MyContextCarrier know that matched my last query?
mycontextport log last --verbose
```

No existing cloud memory service provides this level of injection transparency. They cannot, because revealing exactly what context they've accumulated and how they use it would expose the full extent of the data collection model.

---

## Threat Model: What MyContextCarrier Protects Against

Being precise about what MyContextCarrier protects against, and what it does not, is more honest and more useful than vague privacy claims.

### Protected Against

**Corporate data harvesting.** MyContextCarrier has no server to harvest to. There is no mechanism by which Anthropic, any company, or MyContextCarrier contributors can access your context data.

**Training data extraction.** Your context cannot be used to train any AI model without your explicit action. It is not transmitted. It is not accessible. It cannot be harvested.

**Data broker resale.** Cloud AI memory services operate in a legal environment where user data can be sold, licensed, or transferred in acquisition scenarios. MyContextCarrier data is on your machine and is not subject to any company's data practices.

**Breach of a cloud provider.** If MyContextCarrier had a server and that server was breached, there would be no user context data to expose. The attack surface that cloud providers represent — a single database containing the accumulated context of millions of users — does not exist with MyContextCarrier.

**Legal demands served on a third party.** A subpoena, National Security Letter, or court order served on a cloud provider can compel disclosure of your data without your knowledge. With MyContextCarrier, there is no third party holding your data. A legal demand would have to be served on you directly, which means you are notified and have the ability to respond.

**Passive context leakage across models.** With cloud memory, context accumulated from your interactions with one model may inform the training of another, or be accessible across product lines within a company's ecosystem. With MyContextCarrier, the same context store serves multiple models, but the data never leaves your machine: you are choosing which models receive context from a local system, not allowing a corporation to connect your data across their products without your awareness.

**Account compromise.** There is no MyContextCarrier account to compromise. Your context cannot be accessed by someone who gains access to a MyContextCarrier account because no such account exists.

### Not Protected Against

**Malware on your machine.** If an attacker has code execution on your device, they can potentially access your context store. MyContextCarrier uses encryption to raise the cost of this attack but cannot fully prevent it if your device is compromised. Device security is a prerequisite for MyContextCarrier security.

**Physical device access.** If someone has physical access to your powered-on device and you have not locked your session, they can access the running MyContextCarrier daemon. Standard device security practices apply.

**The AI models themselves.** MyContextCarrier injects your context into prompts sent to AI models — including cloud models. If you choose to inject context into a cloud model like GPT-4, that context leaves your machine as part of the prompt. MyContextCarrier's privacy rules engine allows you to restrict what is injected into cloud versus local models, but the act of injection itself sends data to the model provider. MyContextCarrier is transparent about this. The injection log shows exactly what was sent to which model.

**Legal demands served directly on you.** MyContextCarrier protects your data from third-party legal demands. It does not protect you from legal demands directed at you personally.

---

## Comparison Table

| Property | MyContextCarrier | Cloud Memory (Mem.ai etc.) | App Memory (ChatGPT etc.) | Notes + AI Plugins |
|---|---|---|---|---|
| Data location | Your machine | Provider servers | Provider servers | Your machine (partial) |
| Encryption key holder | You | Provider | Provider | You (if configured) |
| Network requests | None at runtime | Continuous | Continuous | Some |
| Training data risk | None | High | High | Low |
| Cross-model portability | Yes (any MCP model) | No | No | Limited |
| Injection transparency | Full audit log | None | None | Limited |
| Legal demand exposure | Direct only | Third-party exposure | Third-party exposure | Direct only |
| Provider lock-in | None | High | Very high | Medium |
| Automatic context capture | Yes | Yes | Partial | No (manual) |
| Privacy rules engine | Yes | No | No | No |
| Open source | Yes | No | No | Partial |
| Cost | Free | Subscription | Subscription/freemium | Free + plugin costs |
| Context portability | Full export, any format | Limited export | Very limited | Vault export |

---

## On Open Source as a Security Property

It is worth stating explicitly why open source is a security property and not merely a philosophical preference.

**Closed source systems require trust that cannot be verified.** When a company says "we don't sell your data" or "we don't use your data for training," you are trusting a statement by a party with a financial interest in your continued use of their product. The statement may be true. It may not be. You have no way to verify it.

**Open source systems allow trust to be verified.** Every claim MyContextCarrier makes about its behavior — no telemetry, no network requests, local-only data — can be verified by reading the code, building the binary yourself, and monitoring the system's network behavior. You do not have to trust us. You can verify us. This is a categorically different security posture.

**Reproducible builds.** MyContextCarrier is committed to reproducible builds — a property that means anyone who builds the software from source will produce a binary that is cryptographically identical to the binary we distribute. This eliminates the supply chain attack vector where a trusted open source codebase is compiled into an untrustworthy binary.

**The audit surface is public.** Security researchers, privacy advocates, and concerned users can review the codebase without signing NDAs or engaging in responsible disclosure theater. Vulnerabilities found can be reported publicly. Fixes can be reviewed publicly. The security posture of the system is not a secret.

---

## A Note on Future-Proofing

Privacy regulations are tightening globally. GDPR in Europe, CCPA in California, the EU AI Act, and a growing wave of national data protection laws are all moving in the direction of stronger user rights over personal data. Cloud AI memory services are navigating a regulatory environment that may require significant changes to how they collect, store, and use user context data.

MyContextCarrier is structurally compliant with the strongest possible interpretation of these regulations — not because we have designed for compliance, but because the architecture that best serves users also happens to be the architecture that regulators are moving toward requiring. No personal data leaves the user's device. No processing occurs on company infrastructure. No retention policies are needed because no retention occurs outside the user's control.

As regulation tightens, cloud AI memory services will need to build toward what MyContextCarrier already is. MyContextCarrier starts there.

---

## Summary

MyContextCarrier is not more private than cloud alternatives because it has better privacy settings or stronger legal policies. It is more private because the attack surface that enables privacy violations: a server holding your data, a company holding your encryption keys, a training pipeline that can consume your context, does not exist.

The privacy guarantee is architectural. Architecture doesn't lie, change its terms of service, get acquired, or receive National Security Letters.

Your context is on your machine. Your keys are in your hands. Your AI tools get only what you explicitly authorize. Everything else is yours, permanently.

---

*MyContextCarrier — github.com/mycontextport/mycontextport*
*Licensed under MIT*
