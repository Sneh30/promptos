# PROMPTOS — MASTER PROJECT SPECIFICATION GENERATOR (FINAL, GAP-CORRECTED)

You are an elite team of software engineers, compiler engineers, AI researchers, desktop application architects, systems programmers, UI/UX designers, DevOps engineers, product managers, technical writers, security engineers, performance engineers, open-source maintainers, and software testers.

Your task is NOT to build the application immediately.

Your task is to first design the complete engineering specification for a world-class open-source desktop application called **PromptOS**.

Treat this as if PromptOS will become one of the most technically advanced open-source AI desktop applications ever built. Do not simplify. Do not create an MVP. Do not create placeholders. Do not omit sections because they appear too detailed.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PROJECT VISION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

PromptOS is **NOT** another chatbot, wrapper, or prompt editor.

PromptOS is an intelligent **Prompt Compiler**, **Optimizer**, **Analyzer**, **Profiler**, **Linter**, and **AI Control Layer**.

```
User → PromptOS Compiler → Optimized Model-Specific Prompt → Cloud LLM
```

It understands intent, builds an internal representation, runs compiler-style optimization passes, generates a model-specific prompt, predicts quality, estimates cost, detects risk, and produces the highest-quality prompt possible. It does not replace cloud LLMs — it maximizes their effectiveness.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CORE PHILOSOPHY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Human prompts are source code. Compiled prompts are optimized machine code. The compiler must understand intent, preserve semantics, optimize execution, remove redundancy, resolve ambiguity, and improve reasoning, determinism, and reproducibility. Every prompt is *compiled*, never merely rewritten.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
DISTRIBUTION & PLATFORM SCOPE (explicit decision required)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Decide and justify, in Document 2 and 3:

- **v1 scope**: macOS-only (Apple Silicon + Intel) is an acceptable v1 scope for an OSS project, but this must be stated explicitly as a scoping decision, not a silent omission. Document the Windows/Linux roadmap and what architectural choices in v1 keep the door open for it (e.g., avoid macOS-only frameworks in the compiler core; isolate platform-specific code to a thin OS-integration layer).
- **Packaging must include code signing and notarization.** "Drag DMG in, launch, everything works" is only true if the app is signed with a valid Developer ID and notarized by Apple. Specify: Developer ID Application certificate, hardened runtime entitlements, notarization via `notarytool`, stapling, and Gatekeeper behavior on first launch. This is non-negotiable for the stated installation UX and must appear in the SAS release-engineering section.
- Specify the **auto-update mechanism** concretely (e.g., Sparkle framework or an equivalent signed-update-feed approach), including signature verification of update packages.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
EMBEDDED AI REQUIREMENTS (must be concrete, not abstract)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

The local model exists ONLY for prompt compilation (intent extraction, semantic analysis, ambiguity detection, compilation, validation, diagnostics) — never as a substitute for the cloud model.

Specify concretely:

- **Inference runtime choice** with justification vs. alternatives (e.g., llama.cpp for cross-platform CPU/GPU support, MLX for Apple-Silicon-optimized performance, ONNX Runtime for portability). State the trade-offs considered.
- **Model format & quantization strategy** (e.g., GGUF, 4-bit/8-bit quantization) and the target model size, with a stated download/disk-space budget (the DMG size ceiling and the first-run model download size must both be numbers, not adjectives).
- **Hardware acceleration path**: Metal on Apple Silicon, CPU fallback on Intel Macs, minimum RAM requirement.
- **First-run experience**: what happens between "launch" and "ready" — is the model bundled in the DMG (larger download, works fully offline immediately) or fetched on first run (smaller DMG, requires network once)? Pick one and justify it against the "no additional downloads" requirement stated elsewhere — resolve this tension explicitly rather than leaving it implicit.
- **Failure/degradation modes**: what happens if local inference fails to load, if the model file is corrupted, or if the device doesn't meet minimum requirements. Define graceful degradation (e.g., fall back to a lighter rule-based compiler pass) rather than a hard crash.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PROMPT COMPILER
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Design a true compiler pipeline: lexical analysis, parsing, semantic analysis, intent extraction, context extraction, constraint extraction, dependency analysis, contradiction detection, ambiguity detection, context-gap detection, optimization passes, verification, and model-specific code generation.

Treat prompts as structured programs. Build an internal **Prompt AST**. All optimization happens on the AST, never on raw strings.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
MODEL INTELLIGENCE & PROFILE FRESHNESS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Maintain evolving optimization profiles per supported cloud model: context limits, output limits, pricing (including cached-prompt pricing), strengths/weaknesses, reasoning behavior, formatting preferences (XML/Markdown/JSON reliability), coding/writing/math performance, long-context behavior, hallucination tendencies, safety tendencies, tool-usage behavior, structured-output reliability, failure modes.

**Address profile staleness explicitly**: cloud model behavior changes over time (new releases, silent updates). Specify a mechanism for keeping profiles current — e.g., versioned profile bundles shipped with app updates, plus an optional remote profile registry that can be refreshed without a full app update. State how profile accuracy is validated (see Evaluation Framework below) rather than assumed.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
BYOK & API KEY MANAGEMENT (new — was previously only implied)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

- Name the initial set of supported cloud providers explicitly (e.g., Anthropic, OpenAI, Google) rather than leaving "cloud LLM" undefined.
- Specify a provider-abstraction layer so adding a new provider doesn't require touching the compiler core.
- Specify secure key storage using the OS keychain (macOS Keychain Services), never plaintext config files.
- Specify what data leaves the device: only the final compiled prompt (and only when the user explicitly sends it to a cloud model), never the raw draft, analytics, or local-model intermediate representations, unless telemetry is explicitly opted in.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
EVALUATION FRAMEWORK (new — closes the "how do we know it's better" gap)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

An "optimizer" claim is unfalsifiable without a way to measure it. Specify:

- A benchmark suite of representative tasks (coding, writing, analysis, structured extraction) with held-out reference outputs or rubrics.
- An A/B harness that runs both the raw prompt and the compiled prompt against the same target model and scores the difference (quality, cost, latency, determinism).
- How this benchmark suite is used to validate changes to compiler optimization passes and model profiles before they ship (regression testing for prompt quality, analogous to compiler regression tests).

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
LICENSING & GOVERNANCE (new)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

- Choose and justify an OSS license (e.g., Apache 2.0 for patent-grant protection vs. MIT for simplicity) appropriate for a project soliciting external contributions and third-party plugins.
- Define a lightweight governance model (maintainer structure, RFC process for major changes, code of conduct) sufficient for a project aiming at "thousands of contributors."
- Define the **plugin security model**: what runtime plugins execute in (e.g., a sandboxed WASM runtime or an isolated JS runtime with a restricted capability surface), what APIs are exposed, and how a malicious or buggy plugin is prevented from accessing API keys, the filesystem, or arbitrary network endpoints.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
INTERNATIONALIZATION (new)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Specify an i18n/l10n architecture (string externalization, RTL layout support, locale-aware number/date formatting) even if v1 ships English-only — a "flagship" OSS project should not have this bolted on later.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CONCRETE TARGETS (new — replaces vague adjectives with numbers)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

The PRD and SAS must state numeric targets, not descriptions, for at least:
- Cold start time to "ready to compile"
- Prompt compilation latency (local-model pass)
- Idle memory footprint and peak memory footprint
- DMG download size and total on-disk size post-install
- Crash-free session rate target

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
FEATURES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Design in exhaustive detail: Prompt Compiler, Optimizer, Linter, Diff, Version Control, History, Analytics, Token Analytics, Cost Estimator, Latency Estimator, Quality Predictor, Reasoning Predictor, Hallucination Risk Estimator, Context Gap Detector, Auto Clarification Engine, Mission Critical / Deep Analysis / Economy / Balanced modes, Output Length Controller, Privacy Scanner, Secret/API-Key Detection, Sensitive Data Redaction, Plugin System (with the security model above), Compiler Diagnostics, Benchmark Mode (tied to the Evaluation Framework above), Prompt Memory, User Preferences, Settings, Keyboard Shortcuts, Accessibility (state a target WCAG conformance level, e.g., WCAG 2.1 AA, and what that entails for keyboard nav and screen-reader support), Theme Support, Automatic Updates (signed, per distribution section above), Crash Recovery, Logging, optional privacy-respecting Telemetry, Offline Mode (fully defined per the embedded-AI failure-mode section above).

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PRODUCT QUALITY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

The project must feel equivalent in engineering quality to mature open-source software (LLVM, VS Code, Git, Docker, Chromium-caliber decision-making). No shortcuts, no vague architecture, no generic descriptions. Every design decision includes justification and considered alternatives with stated trade-offs.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
YOUR DELIVERABLES — FOUR COMPLETE, SEPARATE DOCUMENTS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**DOCUMENT 1 — MASTER SYSTEM PROMPT**
The complete system prompt defining PromptOS itself: responsibilities, internal reasoning policy, compiler philosophy, optimization strategy, validation strategy, execution modes, model adaptation, safety philosophy, prompt compilation workflow, clarification policy, output generation rules, self-verification, quality guarantees. This is the "brain" of PromptOS.

**DOCUMENT 2 — PRODUCT REQUIREMENTS DOCUMENT (PRD)**
Vision, goals, user personas, user journeys, UX philosophy, functional requirements, non-functional requirements (including the concrete numeric targets above), UI requirements, accessibility (with stated conformance level), i18n scope, feature specifications, settings, analytics, onboarding, performance targets, acceptance criteria, edge cases, platform-scope decision (macOS v1 + roadmap), licensing choice, future roadmap.

**DOCUMENT 3 — SOFTWARE ARCHITECTURE SPECIFICATION (SAS)**
System architecture, compiler architecture, AST, intermediate representation, optimization passes, embedded inference runtime (named and justified), packaging and standalone DMG architecture **including code signing and notarization steps**, update system (named framework), plugin SDK and sandboxing model, storage, caching, analytics engine, privacy layer (data-flow boundary explicitly stated), security, encryption, BYOK/API key management and provider-abstraction layer, testing architecture (including the prompt-quality evaluation harness), logging, diagnostics, CI/CD, release engineering, dependency management, build pipeline, repository structure, coding standards, architecture diagrams described in text, scalability, maintainability, governance model.

**DOCUMENT 4 — IMPLEMENTATION MASTER PROMPT**
A master prompt instructing another frontier LLM to build PromptOS from Documents 1–3: never asks unnecessary questions, makes sensible engineering decisions, implements in logical phases, preserves architectural consistency, maintains coding standards, generates production-ready code, includes testing (including the evaluation harness), documentation, packaging (including signing/notarization), and release engineering. Capable of orchestrating the entire project start to finish.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
FINAL REQUIREMENTS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Do not abbreviate or summarize. Do not skip details because they appear repetitive. Challenge your own architectural decisions; compare alternatives before selecting one; explain trade-offs explicitly. Assume every document becomes the canonical specification for PromptOS. If you discover additional missing requirements while designing, add them proactively and justify why they're necessary. Continue refining until the four documents represent a production-quality engineering specification suitable for a flagship open-source project — including the distribution, evaluation, licensing, and i18n concerns specified above, which are not optional additions but load-bearing parts of the spec.
