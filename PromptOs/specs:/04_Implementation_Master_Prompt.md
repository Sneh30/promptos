# PROMPTOS — IMPLEMENTATION MASTER PROMPT

You are tasked with building **PromptOS** — a world-class open-source desktop prompt compiler application — from scratch, following the specifications in Documents 1 (Master System Prompt), 2 (Product Requirements Document), and 3 (Software Architecture Specification).

You are an elite team of software engineers — all specialties. Your output must be production-quality code. You never ask unnecessary questions. You make sensible engineering decisions. You implement in logical phases. You maintain architectural consistency. You generate testing. You handle packaging. You produce the complete build pipeline.

---

## PROJECT OVERVIEW

PromptOS is a macOS desktop application that:
1. Accepts user-written prompts (natural language)
2. Compiles them through a multi-pass optimization pipeline using local AI analysis
3. Produces optimized, model-specific prompts with diagnostics, cost estimation, and quality metrics
4. Optionally sends compiled prompts to cloud LLMs (Anthropic, OpenAI, Google) via user API keys

**Technical stack**: Rust (compiler core, all services), SwiftUI (UI layer), llama.cpp (local AI inference), Sparkle (auto-updates), WASM (plugin sandbox)

---

## PHASE 1: PROJECT SCAFFOLDING & BUILD SYSTEM

### Tasks
1. Create the repository structure matching the SAS §14 layout:
   ```
   promptos/
   ├── Cargo.toml (workspace)
   ├── crates/
   │   ├── promptos-core/       # Compiler core
   │   ├── promptos-llama/      # llama.cpp FFI bridge  
   │   ├── promptos-profiles/   # Model profiles
   │   ├── promptos-history/    # Storage layer
   │   ├── promptos-plugin/     # Plugin runtime
   │   ├── promptos-provider/   # Cloud provider abstraction
   │   ├── promptos-eval/       # Evaluation harness
   │   └── promptos-keychain/   # Secure key storage
   ├── swift/PromptOSApp/       # SwiftUI application
   └── scripts/                 # Build, sign, notarize scripts
   ```

2. Create the workspace `Cargo.toml` with all crate paths

3. For each crate, create:
   - `Cargo.toml` with plausible dependencies (use the dependency list from SAS §19)
   - `src/lib.rs` with a public module structure and `pub mod` declarations
   - Core type definitions as specified in SAS §2.2 and §2.3

4. Create the SwiftUI project skeleton:
   - `PromptOSApp.swift` — App entry point with `@main` attribute
   - `ContentView.swift` — Three-panel layout container
   - `InputEditorView.swift` — Left panel
   - `OutputView.swift` — Right panel  
   - `DiagnosticsPanelView.swift` — Bottom panel
   - `SettingsView.swift` — Settings window
   - `ToolbarView.swift` — Top toolbar with mode selector and compile button

5. Create `Makefile` with targets: `build`, `test`, `lint`, `release`, `package`, `sign`, `notarize`

6. Create CI workflow `.github/workflows/ci.yml` with lint, test, build steps

### Implementation Requirements
- Use `anyhow` for error handling throughout Rust code
- Use `thiserror` for library-level error types
- Use `tracing` for structured logging (not `log`)
- All public APIs are documented with doc comments
- Modules are `pub mod` in `lib.rs`, types and functions are `pub` only where needed
- Use `#![forbid(unsafe_code)]` in all crates except `promptos-llama`
- Swift code follows MVVM pattern: Views are pure SwiftUI, ViewModels are `@Observable` classes

---

## PHASE 2: COMPILER CORE (promptos-core)

### Tasks

#### 2.1 AST Definition
Implement the complete AST as specified in SAS §2.2:
```rust
// src/ast.rs
pub enum PromptNode { Root(PromptRoot), Section(Section), Block(Block), ... }
pub struct PromptRoot { children: Vec<PromptNode>, annotations: Annotations }
// + all other types from SAS §2.2
```
- All types derive `Debug, Clone, PartialEq, Serialize, Deserialize`
- `SourceSpan` is `{ start: Position, end: Position }` where `Position = { line: usize, column: usize }`
- Implement builder pattern: `PromptRoot::builder()` that returns a `PromptRootBuilder`

#### 2.2 Lexer
Implement the lexer in `src/lexer.rs`:
- Token types: `Instruction`, `Context`, `Constraint`, `FormatSpec`, `RoleSpec`, `Example`, `MetaInstruction`, `Separator`, `Text`, `Heading`, `Newline`, `EOF`
- Token carries `kind: TokenKind`, `span: SourceSpan`, `text: String`
- Lexer is a struct `Lexer` initialized with `&str`, implementing `Iterator<Item = Token>`
- Handle: paragraphs, headings (`# ## ###`), bullet lists, numbered lists, code blocks (```), blockquotes (`>`), horizontal rules (`---`), Markdown formatting

#### 2.3 Parser
Implement the parser in `src/parser.rs`:
- Recursive descent parser
- `Parser { tokens: Vec<Token>, position: usize }`
- `parse() -> Result<PromptRoot>` method
- Produces a tree structure from flat token stream based on Markdown heading hierarchy and block structure
- Error recovery: On parse error, skip to next separator/heading and report diagnostic

#### 2.4 Semantic Analyzer
Implement semantic analyzer interface in `src/semantic.rs`:
```rust
pub trait SemanticAnalyzer {
    fn analyze(&self, ast: &mut PromptRoot) -> Result<Annotations, AnalysisError>;
}
```
- The `LocalModelAnalyzer` implementation:
  - Takes the AST, serializes relevant parts to JSON
  - Calls the local model via `promptos-llama` crate;
  - Receives structured JSON output with intent, ambiguities, contradictions, gaps
  - Merges results into `PromptRoot.annotations`
- The `RuleBasedAnalyzer` implementation (fallback):
  - Pattern-based intent detection (regex for common patterns: "write", "analyze", "extract", "classify")
  - Keyword-based constraint detection ("must", "should", "cannot", "at least", "no more than")
  - Format detection via pattern matching on content
  - Produces lower-confidence annotations

#### 2.5 Optimization Passes
Implement all 10 passes in `src/passes/` as specified in SAS §2.4:

Each pass implements:
```rust
#[async_trait]
pub trait OptimizationPass: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn should_run(&self, mode: CompilationMode, config: &UserConfig) -> bool;
    async fn run(&self, ast: &mut PromptRoot, ctx: &PassContext) -> Result<PassResult, PassError>;
    fn verify(&self, ast: &PromptRoot, original: &PromptRoot) -> Result<(), VerificationFailure>;
}
```

- **PassManager** in `src/pass_manager.rs`:
  - Holds `Vec<Box<dyn OptimizationPass>>`
  - Method `run_all(ast, ctx) -> Vec<PassResult>` that iterates passes, checks `should_run`, executes, verifies, rolls back on verification failure
  - Method `run_selected(ast, ctx, pass_names: &[&str])` for selective execution

#### 2.6 Code Generation
Implement in `src/codegen.rs`:
```rust
pub trait ModelCodeGenerator: Send + Sync {
    fn generate(&self, ast: &PromptRoot, profile: &ModelProfile) -> CompiledPrompt;
    fn model_id(&self) -> ModelId;
}
```
- Default implementation: walk AST depth-first, emit text nodes
- Model-specific overrides (e.g., AnthropicGenerator wraps in `<instructions>` XML tags)
- Structured output schema injection based on profile

#### 2.7 Verification
Implement in `src/verification.rs`:
- `SemanticPreservationChecker`: verify all instructions/constraints from original AST appear in compiled AST (fuzzy match)
- `ContradictionChecker`: verify compiled AST has no contradictions (re-check post-optimization)
- `ContextWindowChecker`: verify compiled prompt fits in target model's context limit with 10% safety margin
- Returns `Vec<VerificationResult>` with status (Pass/Fail/Warning)

#### 2.8 Diagnostics
Implement in `src/diagnostics.rs`:
```rust
pub struct DiagnosticBuilder {
    diagnostics: Vec<Diagnostic>,
}
// Methods: error(), warning(), suggestion(), info()
// All produce structured Diagnostic with severity, code, message, span, recommendation
```

### Implementation Requirements
- All passes are `async` (use `tokio` as the async runtime)
- Pass context `PassContext` provides: `model_profile: &ModelProfile`, `annotations: &Annotations`, `config: &UserConfig`
- Each pass returns `PassResult { tokens_saved: isize, applied: bool, description: String }`
- Test-driven: each pass has unit tests with ~5 test prompts
- Integration tests compile real prompts and verify output

---

## PHASE 3: LLAMA.CPP INTEGRATION (promptos-llama)

### Tasks
1. Create the FFI bridge to llama.cpp:
   - Use `llama-cpp-2` crate (Rust bindings to llama.cpp) or implement custom bindgen-based FFI
   - Bridge exposes: `load_model(path) -> Result<ModelHandle>`, `infer(model, input) -> Result<InferenceOutput>`, `unload_model(handle)`
   - Structured output parsing: model returns JSON, bridge deserializes into structured types

2. Implement model management in `src/model.rs`:
   - `ModelManager` struct managing model lifecycle (load, unload, reload)
   - Model cache: loaded model held in memory, unloaded after 30s of inactivity
   - Integrity check: SHA-256 of model file verified before load

3. Implement inference in `src/inference.rs`:
   - Prompt template wrapping: wraps input in model-specific chat template for structured output
   - Context management: manages KV cache, truncation if input exceeds context
   - Temperature/generation params: deterministic settings for analysis (temperature=0.1, top_p=0.9)

4. Implement fallback in `src/fallback.rs`:
   - `RuleBasedAnalyzer` as pure Rust (no llama.cpp dependency)
   - Pattern-based analysis: regex for intent detection, keyword matching for constraints
   - Returns annotations in the same format as the local model path

### Implementation Requirements
- All llama.cpp C++ interactions must be behind the `ffi` module boundary
- Fallback is always available; escalation to fallback is automatic
- Model loading is async with progress reporting via callback

---

## PHASE 4: PROVIDER ABSTRACTION LAYER (promptos-provider)

### Tasks
1. Implement `ModelProvider` trait as specified in SAS §5.1
2. Implement `AnthropicProvider` using Anthropic Messages API
3. Implement `OpenAIProvider` using OpenAI Chat Completions API
4. Implement `GoogleProvider` using Google Generative AI API
5. Implement `ProviderRegistry`:
   - Holds `HashMap<ProviderId, Box<dyn ModelProvider>>`
   - Methods: `register`, `get`, `providers`, `send_prompt`

### API Implementation Details
- HTTP client: `reqwest` with TLS 1.3
- Rate limiting: token bucket algorithm per provider (configurable RPM)
- Error handling: map provider-specific error responses to unified `ProviderError` enum
- Retry: exponential backoff on 429/5xx errors (max 3 retries)
- Streaming: Support SSE streaming from providers for real-time response display

---

## PHASE 5: STORAGE & HISTORY (promptos-history)

### Tasks
1. Implement `HistoryManager`:
   - Store compilations as MessagePack files in `~/Library/Application Support/com.promptos.app/history/`
   - Index maintained as sorted set of (timestamp, hash) pairs
   - Methods: `save(entry)`, `get(id)`, `list(limit, offset)`, `search(query)`, `delete(id)`, `clear()`
   - Compression: zstd level 3 on each entry

2. Implement `ConfigManager`:
   - Read/write `config.toml` in app support directory
   - Type-safe config struct with serde deserialization
   - Watch file for external changes

---

## PHASE 6: KEYCHAIN INTEGRATION (promptos-keychain)

### Tasks
1. Implement macOS Keychain wrapper using `security-framework` crate:
   - `store_api_key(provider, key)` — creates Keychain item with correct attributes
   - `retrieve_api_key(provider)` — queries and returns key
   - `delete_api_key(provider)` — removes key
   - `key_exists(provider)` — check without retrieving

2. Keychain item attributes:
   - `kSecClass`: `kSecClassGenericPassword`
   - `kSecAttrService`: `"com.promptos.app.api-keys"`
   - `kSecAttrAccount`: Provider ID string
   - `kSecAttrAccessible`: `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`
   - `kSecUseDataProtection`: `true`

3. In-memory key handling:
   - Use `mlock` to prevent swapping
   - Zero memory after use
   - Never log key values

---

## PHASE 7: MODEL PROFILES (promptos-profiles)

### Tasks
1. Create profile data structures matching SAS §7.1
2. Implement `ProfileManager`:
   - Load profiles from `profiles/` directory
   - Validate against JSON Schema
   - Cache in memory (LRU, 50 entries)
   - Refresh from remote registry if enabled
3. Bundle initial profiles for Claude 3.5 Sonnet, GPT-4o, Gemini 1.5 Pro
4. Profile versioning and compatibility checking

---

## PHASE 8: PLUGIN SYSTEM (promptos-plugin)

### Tasks
1. Implement WASM runtime using `wasmtime` crate:
   - `PluginHost` struct managing plugin lifecycle
   - Loading: validate WASM binary, instantiate with sandboxed linker
   - Execution: call plugin hooks (e.g., `on_compile`, `on_diagnostic`)
   - Capability-based security: linker only exposes allowed imports

2. Define plugin ABI:
   - `__promptos_plugin_info() -> PluginInfo` — returns plugin name, version, permissions
   - `__promptos_on_compile(prompt_ast: *const u8, len: u32) -> *const u8` — compilation hook
   - `__promptos_on_diagnostic(diag: *const u8, len: u32)` — diagnostic hook

3. Plugin SDK (`promptos-sdk` crate, in separate repo or subdirectory):
   - Rust crate that compiles to WASM
   - Provides safe wrappers around the ABI functions
   - Helper types: `PluginInfo`, `Diagnostic`, `Span`
   - Example plugin in `plugins/example-plugin/`

---

## PHASE 9: UI LAYER (Swift/SwiftUI)

### Tasks
1. Implement the three-panel layout (SAS §1, PRD §7):
   - `ContentView.swift`: `HSplitView` with left (input), right (output), and bottom (diagnostics) panels
   - Drag handles for resizing panels
   - Remember panel sizes in UserDefaults

2. Implement `InputEditorView.swift`:
   - `NSTextView` wrapped in `NSViewRepresentable` for full IDE-like editing
   - Syntax highlighting (basic: color instructions, constraints, context differently)
   - Line numbers via `LineNumberGutterView`
   - Live token count display

3. Implement `OutputView.swift`:
   - Syntax-highlighted compiled prompt display (read-only)
   - Side-by-side diff view with original (use SwiftUI diff, or implement line-diff)
   - Inline annotations indicating which optimization pass applied each change
   - "Copy" and "Send to Model" buttons

4. Implement `DiagnosticsPanelView.swift`:
   - TabView with tabs: Warnings, Errors, Suggestions, Optimization Report, Risk Assessment
   - Each diagnostic is clickable (scrolls to source location in input editor)
   - Severity color coding
   - Summary header: "3 warnings, 1 error, 5 suggestions"

5. Implement `ToolbarView.swift`:
   - Model picker (dropdown: Claude 3.5 Sonnet, GPT-4o, Gemini 1.5 Pro)
   - Mode picker (segmented control: Economy, Balanced, Deep Analysis, Mission Critical)
   - Compile button (primary action, blue)
   - Send to Model button (secondary, green, disabled if no API key)

6. Implement `SettingsView.swift`:
   - General tab: theme, language, startup behavior
   - Compiler tab: default model, default mode, pass toggles
   - API Keys tab: per-provider key entry (secure text field), validation button
   - Privacy tab: telemetry opt-in toggle with clear disclosure
   - Advanced tab: local model toggle, logging level

7. Implement window management:
   - Main window: `WindowGroup` with `ContentView`
   - Settings window: `Settings` scene with `SettingsView`
   - About window: standard macOS about panel
   - Menu bar: File (New, Open, Save, Export), Edit (standard), View (Toggle panels), Compile (Compile, Send), Window, Help

8. Implement theming:
   - Dark and light variants
   - Follow system appearance by default
   - Override via settings
   - All colors defined as `Color` assets in asset catalog

9. Implement accessibility (WCAG 2.1 AA):
   - All interactive elements have `accessibilityLabel` and `accessibilityHint`
   - Focus management: tab order follows visual layout
   - Keyboard navigation: Tab/Shift+Tab between panels, arrow keys within panels
   - `@FocusState` bindings for programmatic focus management
   - Dynamic Type support for text sizing

---

## PHASE 10: PACKAGING & DISTRIBUTION (scripts/)

### Tasks
1. Create `scripts/build.sh`:
   - Build Rust crates for both arm64 and x86_64
   - Create universal binary with `lipo`
   - Build SwiftUI app
   - Copy Rust dylib/static lib into .app bundle
   - Copy bundled model GGUF into `.app/Contents/Resources/`
   - Copy profiles into `.app/Contents/Resources/profiles/`

2. Create `scripts/sign.sh`:
   - `codesign --deep --force --verify-verbose --options runtime --timestamp --entitlements promptos.entitlements`
   - Sign all frameworks (Sparkle, etc.)
   - Verify with `codesign --verify --deep --strict`

3. Create `scripts/notarize.sh`:
   - `xcrun notarytool submit --apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" --team-id "$TEAM_ID" --wait`
   - `xcrun stapler staple`
   - `spctl --assess --verbose --type exec`

4. Create `scripts/package.sh`:
   - Build DMG with symlink to /Applications
   - Custom background image
   - Volume name: "PromptOS"
   - Window size and icon positioning

5. Create `scripts/release.sh`:
   - Orchestrate full release pipeline: build → test → sign → notarize → package → upload to GitHub Releases
   - Update appcast.xml for Sparkle

---

## PHASE 11: EVALUATION FRAMEWORK (promptos-eval)

### Tasks
1. Implement `BenchmarkSuite`:
   - Load benchmark prompts from `benchmarks/` directory (JSON/TOML format)
   - Each benchmark: `{ id, category, prompt, expected_output, scoring_rubric }`
   - Run each prompt through the compiler pipeline
   - Score compiled output against rubric

2. Implement `ABHarness`:
   - `run_ab_test(raw_prompt, compiled_prompt, model, key) -> ABTestResult`
   - Calls the cloud model with both raw and compiled versions
   - Scores outputs using built-in rubrics or LLM-as-judge
   - Reports all metrics: quality, cost, latency, determinism

3. Implement `RegressionChecker`:
   - `compare(baseline: EvalResults, current: EvalResults) -> RegressionReport`
   - Flags any category with >2% degradation
   - Blocks CI if regression detected

4. Create benchmark prompts:
   - 50 code generation prompts (various languages, difficulty levels)
   - 50 writing/analysis prompts
   - 30 structured extraction prompts (JSON, CSV, markdown tables)
   - 30 classification/reasoning prompts
   - 40 instruction-following prompts (multi-constraint)

---

## PHASE 12: TESTING

### Requirements
- **Unit tests**: Every Rust function has a test; aim for >90% line coverage
- **Integration tests**: Each pass has 5+ integration tests in `tests/` with real prompt inputs and expected AST/output
- **Evaluation tests**: Benchmark suite runs as part of CI; regression check is blocking
- **UI tests**: XCTest UI tests for critical paths: compile, mode switch, theme toggle, settings
- **Edge case tests**: Empty input, single word, 100K tokens, binary input, unicode, RTL text, extremely nested markdown

### Test directories
```
crates/promptos-core/tests/
├── integration/
│   ├── test_lexer.rs
│   ├── test_parser.rs
│   ├── test_semantic.rs
│   ├── test_passes/
│   │   ├── test_redundancy.rs
│   │   ├── test_ambiguity.rs
│   │   └── ...
│   ├── test_codegen.rs
│   ├── test_verification.rs
│   └── test_full_pipeline.rs
├── fixtures/
│   ├── prompts/           # Sample input prompts
│   ├── asts/              # Expected AST outputs
│   ├── compiled/          # Expected compiled outputs
│   └── profiles/          # Test model profiles
└── benchmarks/
    ├── code_generation.json
    ├── writing_analysis.json
    ├── structured_extraction.json
    ├── classification.json
    └── instruction_following.json
```

---

## IMPLEMENTATION ORDER (DEPENDENCY GRAPH)

```
Phase 1 (Scaffolding)
    │
    ▼
Phase 2 (Compiler Core)
    │
    ├────────────────────────────────────┐
    ▼                                    ▼
Phase 3 (llama.cpp bridge)      Phase 4 (Provider Abstraction)
    │                                    │
    ▼                                    │
Phase 5 (Storage) ◄──────────────────────┤
    │                                    │
    ▼                                    ▼
Phase 6 (Keychain)              Phase 7 (Model Profiles)
    │                                    │
    ▼                                    ▼
Phase 8 (Plugin System)         Phase 9 (UI Layer) ◄─────── Phase 5,6,7,4
    │                                    │
    ▼                                    ▼
Phase 10 (Packaging)            Phase 11 (Evaluation)
    │                                    │
    └────────────────┬───────────────────┘
                     ▼
              Phase 12 (Testing & Integration) ◄────── All phases
                     │
                     ▼
              Release Build
```

---

## DESIGN PRINCIPLES TO FOLLOW

1. **No unnecessary questions.** If a decision is not specified in the docs, make a sensible default and document it in a comment.
2. **Production-quality code.** Error handling on every fallible operation. No `unwrap()` or `expect()` in production code (only in tests).
3. **Performance matters.** Profile early. The compilation latency budget is <2s. If the local model takes longer, the issue is in prompt engineering or model selection, not the compiler pipeline.
4. **Security by default.** API keys never logged. Network requests only to configured endpoints. Plugin sandbox enforced.
5. **Privacy-preserving.** No data exfiltration. Telemetry opt-in. Crash reports without prompt content.
6. **Test everything.** Every pass has tests. Every provider has integration tests. Benchmark suite is the source of truth for quality claims.
7. **Architectural consistency.** No framework changes mid-project. If a pattern is established in Phase 2, it continues through Phase 12.
8. **Forward compatibility.** The compiler core is pure Rust with no platform-specific deps. Cross-platform ports require only UI and OS-integration changes.

---

## ACCEPTANCE CRITERIA FOR THIS IMPLEMENTATION

1. All crates compile without errors (`cargo build --workspace`)
2. All tests pass (`cargo test --workspace`)
3. CLI evaluation tool can compile a prompt and produce output
4. Model profiles load and validate correctly
5. Provider abstraction can authenticate and send a prompt (unit-tested with mock HTTP)
6. History can save, load, and search entries
7. Keychain store/retrieve/delete roundtrip works
8. Plugin system can load a WASM binary and call hooks
9. SwiftUI app launches and displays the three-panel layout
10. "Compile" flow from input text to compiled output display works end-to-end
11. Benchmark suite runs and produces scores (doesn't need to pass thresholds yet)
12. Build/packaging scripts produce a signed (or dev-signed) .app bundle

---

## FINAL INSTRUCTIONS

Generate the complete, production-ready codebase for PromptOS. Every file, every function, every test. The code should be immediately compilable with `cargo build --workspace` and `xcodebuild`.

Do not abbreviate implementations. Do not use `todo!()` or `unimplemented!()` in production code. Every function has a real implementation.

When you encounter a specification that could be interpreted in multiple ways, choose the interpretation that maximizes: (1) user privacy, (2) code quality, (3) performance, (4) maintainability, (5) cross-platform readiness — in that order.

Begin.
