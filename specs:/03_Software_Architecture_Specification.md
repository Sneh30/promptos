# PROMPTOS — SOFTWARE ARCHITECTURE SPECIFICATION (SAS)

**Version**: 1.0
**Status**: Final

---

## 1. SYSTEM ARCHITECTURE OVERVIEW

```
┌─────────────────────────────────────────────────────────────────┐
│                     PROMPTOS APPLICATION                        │
│                                                                 │
│  ┌──────────┐  ┌──────────────────┐  ┌──────────────────────┐  │
│  │  UI       │  │  COMPILER CORE   │  │  INFERENCE ENGINE   │  │
│  │  Layer   │◄─┤  (Rust)           │◄─┤  (llama.cpp)        │  │
│  │  (Swift)  │  │                  │  │                     │  │
│  │          │  │  • Lexer/Parser   │  │  • GGUF Model Load  │  │
│  │  • Input  │  │  • AST Builder   │  │  • Metal GPU Inf.   │  │
│  │  • Output │  │  • Semantic Anal │  │  • CPU Fallback     │  │
│  │  • Diff   │  │  • Opt Passes    │  │  • Structured Out   │  │
│  │  • Diag.  │  │  • Code Gen      │  │  • Context Mgmt     │  │
│  └──────────┘  │  • Verification   │  └──────────────────────┘  │
│                │  • Eval Harness   │                            │
│                └──────────────────┘                            │
│                        │                                       │
│                        ▼                                       │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                 SERVICE LAYER                            │  │
│  │  ┌────────┐ ┌──────────┐ ┌────────┐ ┌───────────────┐  │  │
│  │  │ Keychain│ │ Profiles │ │ History│ │ Plugin Runtime│  │  │
│  │  │ Manager│ │ Registry │ │ Manager│ │ (WASM)        │  │  │
│  │  └────────┘ └──────────┘ └────────┘ └───────────────┘  │  │
│  └─────────────────────────────────────────────────────────┘  │
│                        │                                       │
│                        ▼                                       │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                 NETWORK LAYER                            │  │
│  │  ┌──────────┐ ┌────────────┐ ┌────────┐ ┌───────────┐  │  │
│  │  │ Provider  │ │ Profile    │ │ Update  │ │ Telemetry │  │  │
│  │  │ API Abst.│ │ Registry   │ │ Manager │ │ (opt-in)  │  │  │
│  │  └──────────┘ └────────────┘ └────────┘ └───────────┘  │  │
│  └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Architectural Style
- **Pattern**: Hexagonal Architecture (Ports & Adapters) with CQRS for the compiler pipeline
- **Language**: Rust for compiler core, Swift for UI layer, C++ for llama.cpp bridge
- **Concurrency**: Actor-based model for service layer; thread pool for compiler pipeline; async for network I/O

### Layer Responsibilities

| Layer | Technology | Responsibility |
|---|---|---|
| UI Layer | SwiftUI + AppKit | All user-facing interface, rendering, animations |
| Compiler Core | Rust (compiled as static library via C-ABI) | Lexing, parsing, AST, optimization, code generation, verification |
| Inference Engine | llama.cpp (C++ via Rust FFI) | Local model loading, inference, structured output parsing |
| Service Layer | Rust | Keychain access, profile management, history persistence, plugin runtime |
| Network Layer | Rust (async, tokio) | Provider API abstraction, profile sync, automatic updates, opt-in telemetry |

---

## 2. COMPILER ARCHITECTURE

### 2.1 Compiler Pipeline (Detailed)

```
Raw Prompt (String)
    │
    ▼
┌──────────────────────┐
│  1. Lexer            │
│  ┌────────────────┐  │
│  │ TokenStream     │  │  Token types: Instruction, Context, Constraint,
│  │                 │  │  Format, Role, Example, Meta, Separator, Unknown
│  └────────────────┘  │
└──────────────────────┘
    │
    ▼
┌──────────────────────┐
│  2. Parser           │
│  ┌────────────────┐  │
│  │ Raw AST         │  │  Tree structure: PromptNode → SectionNode → BlockNode
│  │                 │  │  Each node has type, span (line:col range), children
│  └────────────────┘  │
└──────────────────────┘
    │
    ▼
┌──────────────────────┐
│  3. Semantic Analysis │  ◄── Local Model Inference
│  ┌────────────────┐  │
│  │ Annotated AST   │  │  Annotations: intent, context, constraints
│  │                 │  │  Dependencies, contradictions, ambiguities, gaps
│  └────────────────┘  │
└──────────────────────┘
    │
    ▼
┌──────────────────────┐
│  4. Optimization     │
│  ┌────────────────┐  │
│  │ Optimized AST   │  │  Multi-pass: each pass walks the AST and transforms
│  │                 │  │  PassManager orchestrates passes in order
│  └────────────────┘  │
└──────────────────────┘
    │
    ▼
┌──────────────────────┐
│  5. Code Generation  │  ◄── Target Model Profile
│  ┌────────────────┐  │
│  │ Compiled Prompt │  │  Model-specific formatting, instruction style
│  │ (String)       │  │  Output structure adaptation
│  └────────────────┘  │
└──────────────────────┘
    │
    ▼
┌──────────────────────┐
│  6. Verification     │
│  ┌────────────────┐  │
│  │ Verified Output │  │  Semantic preservation check
│  │                 │  │  Contradiction check (post-opt)
│  │                 │  │  Context window compliance
│  │                 │  │  Format validity check
│  └────────────────┘  │
└──────────────────────┘
    │
    ▼
Compiled Prompt + Diagnostics + Metrics + Diff
```

### 2.2 Prompt AST Specification

```rust
enum PromptNode {
    Root(PromptRoot),
    Section(Section),
    Block(Block),
    Instruction(Instruction),
    Context(Context),
    Constraint(Constraint),
    FormatSpec(FormatSpec),
    RoleSpec(RoleSpec),
    Example(Example),
    MetaInstruction(MetaInstruction),
}

struct PromptRoot {
    children: Vec<PromptNode>,
    annotations: Annotations,
}

struct Section {
    heading: String,
    level: u8,
    children: Vec<PromptNode>,
    span: SourceSpan,
}

struct Block {
    block_type: BlockType, // Paragraph, List, Code, Quote, Table
    content: String,
    children: Vec<PromptNode>,
    span: SourceSpan,
}

struct Instruction {
    verb: InstructionVerb, // Generate, Analyze, Extract, Classify, etc.
    object: String,
    modifiers: Vec<Modifier>,
    confidence: f32, // How confident the analyzer is this is an instruction
    span: SourceSpan,
}

struct Context {
    content: String,
    context_type: ContextType, // Background, Definition, Reference, Data
    relevance_score: f32,
    span: SourceSpan,
}

struct Constraint {
    constraint_type: ConstraintType, // Positive, Negative, Length, Format, Content, Time
    value: String,
    severity: ConstraintSeverity, // Required, Preferred, Suggested
    span: SourceSpan,
}

// Additional node types follow the same pattern
```

### 2.3 Annotations Structure

```rust
struct Annotations {
    // Semantic Analysis Results
    intent: Option<Intent>,
    detected_ambiguities: Vec<Ambiguity>,
    detected_contradictions: Vec<Contradiction>,
    detected_context_gaps: Vec<ContextGap>,
    dependencies: Vec<Dependency>,
    
    // Optimization Results
    optimization_log: Vec<OptimizationEntry>,
    token_count_original: usize,
    token_count_compiled: usize,
    estimated_cost_original: f64,
    estimated_cost_compiled: f64,
    estimated_latency_original: f64,
    estimated_latency_compiled: f64,
    quality_score_original: f32,
    quality_score_compiled: f32,
    hallucination_risk: f32,
    
    // Diagnostics
    diagnostics: Vec<Diagnostic>,
    compiler_version: String,
    timestamp: u64,
}

struct Intent {
    primary_task: String,
    domain: Option<String>,
    output_type: OutputType, // Text, Code, JSON, Structured, ImageDesc, etc.
    complexity: Complexity, // Simple, Moderate, Complex, VeryComplex
}

struct Ambiguity {
    text: String,
    span: SourceSpan,
    interpretations: Vec<String>,
    recommended_resolution: Option<String>,
    confidence: f32,
}

struct Contradiction {
    constraint_a: SourceSpan,
    constraint_b: SourceSpan,
    description: String,
}

struct ContextGap {
    gap_type: GapType, // MissingInstruction, MissingContext, MissingFormat, MissingRole
    description: String,
    suggested_addition: Option<String>,
}

struct OptimizationEntry {
    pass_name: String,
    pass_version: String,
    status: PassStatus, // Applied, Skipped, Failed, Reverted
    tokens_saved: isize,
    description: String,
    span: Option<SourceSpan>,
}

struct Diagnostic {
    severity: DiagnosticSeverity, // Error, Warning, Suggestion, Info
    message: String,
    span: Option<SourceSpan>,
    diagnostic_code: String,
    recommendation: Option<String>,
}
```

### 2.4 Optimization Passes (Detailed)

#### Pass 1: Redundancy Elimination
- **Algorithm**: Walk AST, compute semantic similarity between instruction/context/constraint pairs using local-model embeddings. If similarity > threshold (0.85), merge or remove duplicate.
- **Token impact**: 5-15% reduction typical
- **Failure mode**: If local model unavailable, use Jaccard similarity on token sets (less accurate but functional)

#### Pass 2: Ambiguity Resolution
- **Algorithm**: For each detected ambiguity, attempt resolution using context. If one interpretation has significantly higher contextual support, auto-resolve. Otherwise, flag for user.
- **Token impact**: No reduction (may add clarification), quality improvement
- **Failure mode**: Always flag unresolved ambiguities; never silently choose

#### Pass 3: Context Window Optimization
- **Algorithm**: Score each AST node by relevance-to-intent. Prune or compress nodes below threshold. Reorder by priority score descending.
- **Token impact**: 10-25% reduction in long prompts
- **Failure mode**: Never prune content above user-set retention threshold

#### Pass 4: Instruction Strengthening
- **Algorithm**: Match weak instruction patterns ("could you", "maybe", "I'd like if") against known weak→strong mapping. Apply deterministic text transformations.
- **Token impact**: 2-5% reduction, quality improvement
- **Failure mode**: Conservative mapping only; never change instruction semantics

#### Pass 5: Format Normalization
- **Algorithm**: Map target model's format reliability data to format spec in AST. Transform all format instructions to the model's most reliable format (XML for Claude, Markdown for GPT, etc.).
- **Token impact**: Variable
- **Failure mode**: If model format preference unknown, preserve original format

#### Pass 6: Token Budget Optimization
- **Algorithm**: Apply model-specific tokenization estimate to each node. Reorder/compress to fit within context budget with 10% safety margin for output tokens.
- **Token impact**: 5-20% reduction enforced
- **Failure mode**: If composition is impossible, error diagnostic recommending user truncation

#### Pass 7: Prioritization Ordering
- **Algorithm**: Reorder top-level AST children by: constraints > instructions > context > examples > meta-instructions. Within each category, order by estimated importance.
- **Token impact**: No reduction, quality improvement
- **Failure mode**: Always reorder; original order preserved in metadata

#### Pass 8: Chain-of-Thought Scaffolding
- **Algorithm**: If task complexity is "Complex" or "VeryComplex" and target model has strong CoT capability (per profile), inject minimal CoT framing: "Let's work through this step by step:" or model-specific equivalent.
- **Token impact**: 5-15 token increase
- **Failure mode**: Never inject CoT for models with poor CoT reliability

#### Pass 9: Persona Reinforcement
- **Algorithm**: If RoleSpec detected, strengthen persona framing with model-appropriate role prompt syntax (XML role tags for Claude, system message for OpenAI).
- **Token impact**: 5-20 token increase
- **Failure mode**: If role not detected, skip

#### Pass 10: Few-Shot Example Optimization
- **Algorithm**: Deduplicate examples by semantic similarity; trim examples to minimum viable length; order by relevance to task; remove examples with conflicting output patterns.
- **Token impact**: 10-40% reduction on example-heavy prompts
- **Failure mode**: If fewer than 2 examples remain, warn user

### 2.5 PassManager

```rust
struct PassManager {
    passes: Vec<Box<dyn OptimizationPass>>,
    mode: CompilationMode,
    target_model: ModelId,
    user_config: UserConfig,
}

trait OptimizationPass {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn should_run(&self, mode: CompilationMode, config: &UserConfig) -> bool;
    fn run(&self, ast: &mut PromptRoot, ctx: &PassContext) -> Result<PassResult, PassError>;
    fn verify(&self, ast: &PromptRoot, original: &PromptRoot) -> Result<(), VerificationFailure>;
}
```

---

## 3. EMBEDDED INFERENCE RUNTIME (llama.cpp)

### 3.1 Runtime Selection: llama.cpp (GGUF)

**Decision**: llama.cpp is the embedded inference runtime for the local model.

**Alternatives Considered**:

| Runtime | Strengths | Weaknesses | Verdict |
|---|---|---|---|
| **llama.cpp** | Cross-platform (macOS/Linux/Windows), CPU+GPU (Metal, CUDA, Vulkan), GGUF ecosystem, mature OSS, active maintenance, Intel Mac support via CPU | Slightly lower peak throughput than MLX on Apple Silicon | **Selected** |
| MLX | Highest perf on Apple Silicon (MPS), Apple-native | Apple Silicon only, no Intel Mac, no Windows/Linux, rapidly changing API, younger project | Rejected — v1 is macOS but must support Intel Mac and keep cross-platform door open |
| ONNX Runtime | Broad hardware support, Microsoft-backed | Higher overhead, less optimized for local LLM inference, more complex integration | Rejected — overkill for single-model local inference |
| Core ML | Apple-native, Metal optimized | Model conversion complexity, Apple Silicon only, less flexible runtime, no cross-platform potential | Rejected — same Intel Mac issue as MLX, plus harder to bundle custom GGUF models |

**Justification**: llama.cpp is the only runtime that simultaneously satisfies: (1) Apple Silicon Metal acceleration, (2) Intel Mac CPU fallback, (3) cross-platform architecture for Windows/Linux v2, (4) mature GGUF model format with broad quantization options, (5) proven stability and performance in production OSS desktop applications.

### 3.2 Model Format & Quantization

| Parameter | Value | Rationale |
|---|---|---|
| **Format** | GGUF | Standard format for llama.cpp; supports all quantization types |
| **Quantization** | Q4_K_M | Best quality-to-size ratio for a desktop application; ~4.5 bits/weight |
| **Base Model Size** | ~1.5B parameters | Sufficient for structured prompt analysis tasks; small enough for fast inference on Intel Macs |
| **Quantized Size** | ~220 MB | Fits within 285 MB DMG budget alongside app |
| **Context Length** | 4096 tokens | Sufficient for analyzing any prompt that fits in a cloud model's context |
| **Embedding Dimension** | 1536 | Compatible with semantic similarity thresholds used in optimization passes |

### 3.3 Hardware Acceleration Path

| Hardware | Acceleration | Expected Performance |
|---|---|---|
| Apple M1 | Metal GPU (MPS backend) | <1.5s for 4K token analysis |
| Apple M2/M2 Pro/Max | Metal GPU | <1s for 4K token analysis |
| Apple M3+ | Metal GPU (dynamic cache) | <0.8s for 4K token analysis |
| Intel Mac (any) | CPU fallback (no Intel GPU compute) | <4s for 4K token analysis |
| All (model load) | Metal GPU for context ~4096 | First load <5s, subsequent <2s |

### 3.4 Minimum System Requirements

| Component | Minimum | Recommended |
|---|---|---|
| RAM | 8 GB | 16 GB |
| Storage (free) | 2 GB | 5 GB |
| macOS | 13 Ventura | 14 Sonoma+ |
| GPU | Metal-capable (integrated or discrete) | Metal 3-capable |

### 3.5 llama.cpp Integration Architecture

```
┌─────────────────────────┐
│   Rust Compiler Core     │
│                         │
│   llama_bridge.rs       │
│   ┌─────────────────┐   │
│   │ FFI to C API    │   │
│   └─────────────────┘   │
│         │                │
│         ▼                │
│   llama_sys crate        │
│   (bindgen-generated)    │
└─────────────────────────┘
         │
         ▼
┌─────────────────────────┐
│   llama.cpp (C++)        │
│   ┌─────────────────┐   │
│   │ metal.cpp       │   │  Apple GPU kernel compilation
│   │ llama.cpp       │   │  Core inference engine
│   │ ggml.c          │   │  Tensor computation library
│   │ ggml-metal.m    │   │  Metal GPU backend (ObjC++)
│   └─────────────────┘   │
│                         │
│   Built as:             │
│   libllama.a (static)   │
└─────────────────────────┘
```

### 3.6 Failure & Degradation Modes

| Failure Mode | Detection | Degradation Behavior |
|---|---|---|
| Model file corrupted | SHA-256 mismatch on load | Show error dialog with "Re-download model" option; fall back to rule-based pass immediately |
| GPU inference fails | Metals GPU returns error code | Automatic fallback to CPU inference; show non-blocking notification |
| Model fails to load | llama.cpp returns null context | Log error; fall back to rule-based pass; show warning badge |
| Out of memory during inference | `ggml_malloc` returns null | Show "Low memory" warning; trigger garbage collection for any cached inference results; degrade gracefully |
| Model file not found | File I/O error on bundle path | Trigger automatic model extraction from bundle; if bundle missing, show "Reinstall app" dialog |
| CPU inference too slow on Intel | Elapsed time >5s threshold | Suggest enabling GPU acceleration if available; otherwise show performance advisory |

### 3.7 Rule-Based Fallback Compiler

When the local model is unavailable, the compiler falls back to a lighter pipeline:

```
┌────────────────────┐
│  Lexical Analysis  │  (same as full pipeline)
└────────────────────┘
         │
         ▼
┌────────────────────┐
│  Pattern Matching  │  Rule-based intent detection via regex patterns
│  (Rule-based)     │  Constraint extraction via keyword matching
│                   │  Format detection via structure heuristics
│                   │  Ambiguity detection limited to known patterns
└────────────────────┘
         │
         ▼
┌────────────────────┐
│  Optimization (sub)│   - Redundancy elimination (Jaccard similarity)
│                   │   - Format normalization
│                   │   - Instruction strengthening (limited pattern map)
│                   │   - Token budget optimization
└────────────────────┘
         │
         ▼
┌────────────────────┐
│  Code Generation   │  (same as full pipeline)
│  Model-Specific    │
└────────────────────┘
```

The rule-based fallback produces ~60% of the quality improvement of the full pipeline but completes in <200ms. All diagnostics are downgraded to "suggestion" severity since they lack the confidence of LLM-based analysis.

---

## 4. DATA FLOW & PRIVACY LAYER

### 4.1 Data Flow Diagram

```
User Input (Raw Prompt)
    │
    │  LOCAL ONLY — never leaves device
    ▼
┌──────────────────────┐
│  Compiler Core        │
│  ┌────────────────┐   │
│  │ Local Analysis  │   │  ◄── Local model (on-device)
│  │ Optimization    │   │
│  │ Diagnostics     │   │
│  └────────────────┘   │
└──────────────────────┘
    │
    │  Compiled Prompt (only when user clicks "Send")
    ▼
┌──────────────────────┐
│  Network Layer        │
│  ┌────────────────┐   │
│  │ Provider API    │   │  ◄── Cloud LLM (Anthropic/OpenAI/Google)
│  │ Abstraction     │   │
│  └────────────────┘   │
└──────────────────────┘
    │
    ▼
Cloud Model Response  ──►  Displayed to user
```

### 4.2 Data Boundary

| Data Element | Stored Locally | Transmitted to Cloud | Stored in Telemetry (opt-in) |
|---|---|---|---|
| Raw prompt (draft) | Yes (until history trimmed) | Never | Never |
| Compiler AST / IR | Yes (in memory during compilation) | Never | Never |
| Local model analysis output | Yes (in memory during compilation) | Never | Never |
| Compiled prompt | Yes (in history) | Only on user "Send" action | Never |
| Cloud model response | Yes (in history with compiled prompt) | N/A | Never |
| Diagnostics & metrics | Yes (in history) | Never | Aggregate only (no prompt content) |
| API keys | macOS Keychain only | To configured API endpoints during use | Never |
| Compilation statistics (anonymized) | Yes (local analytics) | Only if telemetry opted in | Yes (no prompt content, no model outputs) |
| Crash reports | Yes (local logs) | Only if crash reporting opted in | With user consent, no prompt content |

---

## 5. PROVIDER ABSTRACTION LAYER

### 5.1 Architecture

```rust
trait ModelProvider {
    fn id(&self) -> ProviderId;
    fn name(&self) -> &str;
    fn supported_models(&self) -> Vec<ModelId>;
    fn send_prompt(&self, prompt: &CompiledPrompt, key: &ApiKey) -> Result<ModelResponse, ProviderError>;
    fn estimate_cost(&self, prompt: &CompiledPrompt, model: &ModelId) -> CostEstimate;
    fn validate_key(&self, key: &ApiKey) -> Result<bool, ProviderError>;
}

struct CompiledPrompt {
    text: String,
    model_id: ModelId,
    mode: CompilationMode,
    max_output_tokens: Option<u32>,
    temperature: Option<f32>,
    structured_output_schema: Option<JsonSchema>,
}

struct ModelResponse {
    text: String,
    model_id: ModelId,
    input_tokens: u32,
    output_tokens: u32,
    latency_ms: u64,
    finish_reason: FinishReason,
    raw_response: serde_json::Value,
}
```

### 5.2 Initial Providers

| Provider | Models | API Base URL | Auth Method |
|---|---|---|---|
| Anthropic | Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku | `https://api.anthropic.com/v1/` | `x-api-key` header |
| OpenAI | GPT-4o, GPT-4o-mini, o1, o3 | `https://api.openai.com/v1/` | `Authorization: Bearer` header |
| Google | Gemini 1.5 Pro, Gemini 1.5 Flash | `https://generativelanguage.googleapis.com/v1/` | `API-Key` query param or `Authorization: Bearer` |

### 5.3 Adding a New Provider

1. Implement the `ModelProvider` trait
2. Register the provider in the `ProviderRegistry`
3. Add model profile(s) to the profiles directory
4. Compile — no changes to compiler core required

---

## 6. API KEY MANAGEMENT

### 6.1 Secure Storage (macOS Keychain)

```rust
struct KeychainManager {
    service_name: String, // "com.promptos.app"
}

impl KeychainManager {
    fn store_api_key(provider: ProviderId, key: ApiKey) -> Result<(), KeychainError>;
    fn retrieve_api_key(provider: ProviderId) -> Result<Option<ApiKey>, KeychainError>;
    fn delete_api_key(provider: ProviderId) -> Result<(), KeychainError>;
    fn has_key(provider: ProviderId) -> Result<bool, KeychainError>;
}
```

- **Storage**: `kSecClassGenericPassword` with access control `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`
- **Encryption**: Keychain handles encryption at rest; data is never stored outside Keychain
- **In-memory**: Keys are held in locked memory (`mlock`) during use, zeroed after request completes
- **UI**: Key setup via native credential input fields (secure text entry); key masking in settings

---

## 7. MODEL PROFILE SYSTEM

### 7.1 Profile Format

```toml
[profile]
model_id = "claude-3.5-sonnet"
provider = "anthropic"
version = "1.2.0"
last_updated = "2026-01-15"

[specs]
context_limit_input = 200000
context_limit_output = 8192
max_output_tokens = 8192
pricing_input_per_mtok = 3.00
pricing_output_per_mtok = 15.00
pricing_cached_input_per_mtok = 0.30

[performance]
reasoning_quality = 0.92
coding_quality = 0.89
writing_quality = 0.90
analysis_quality = 0.91
structured_extraction_quality = 0.88
tool_use_reliability = 0.85
instruction_following = 0.93

[format_reliability]
xml = 0.95
markdown = 0.90
json_structured = 0.85
json_unstructured = 0.75

[behavior]
chain_of_thought_effective = true
self_correction_tendency = 0.3
verbose_conciseness = 0.5
long_context_retrieval_accuracy = 0.85
primacy_effect_bias = 0.2
recency_effect_bias = 0.3

[safety]
hallucination_risk_general = 0.08
hallucination_risk_coding = 0.05
hallucination_risk_factual = 0.12
refusal_rate = 0.02
sycophancy_tendency = 0.3
```

### 7.2 Profile Registry

- **Local registry**: Bundled profiles in `~/Library/Application Support/com.promptos.app/profiles/`
- **Remote registry**: Hosted at `https://profiles.promptos.app/v1/` (optional)
- **Update mechanism**: Check remote for newer profile versions on launch (if enabled); download and verify signature; replace local copy
- **Versioning**: Profiles use semver; compiler checks profile compatibility before loading
- **Validation**: Profiles are verified against a JSON Schema and SHA-256 signed by the PromptOS team

---

## 8. STORAGE & CACHING

### 8.1 Storage Layout

```
~/Library/Application Support/com.promptos.app/
├── profiles/
│   ├── anthropic-claude-3.5-sonnet-1.2.0.toml
│   ├── openai-gpt-4o-1.1.0.toml
│   └── google-gemini-1.5-pro-1.0.0.toml
├── history/
│   ├── index.msgpack          # Sorted list of (timestamp, hash) entries
│   ├── entries/               # Individual prompt entries (msgpack)
│   │   ├── a1b2c3d4.msgpack
│   │   └── ...
│   └── vindex.msgpack         # Version index for forward compatibility
├── plugins/                   # WASM plugin binaries
│   └── ...
├── config.toml                # User preferences (no secrets)
├── telemetry.db               # Opt-in analytics (SQLite, encrypted at rest)
└── crash_reports/             # Opt-in crash reports
    └── ...
```

### 8.2 Prompt History Storage

```rust
struct HistoryEntry {
    id: Uuid,
    timestamp: u64,
    user_prompt: String,
    compiled_prompt: String,
    target_model: ModelId,
    mode: CompilationMode,
    diagnostics: Vec<Diagnostic>,
    metrics: CompilationMetrics,
    diff: Diff,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

struct CompilationMetrics {
    token_count_original: u32,
    token_count_compiled: u32,
    estimated_cost: f64,
    estimated_latency_ms: u64,
    quality_score: f32,
    hallucination_risk: f32,
    passes_applied: Vec<String>,
    compilation_time_ms: u64,
}
```

- **Storage format**: MessagePack (compact binary serialization)
- **Retention**: Last 100 compilations by default, configurable up to 1000
- **Compression**: Individual entries compressed with zstd (level 3)
- **Indexing**: Sorted set index for efficient range queries and search

---

## 9. PLUGIN SYSTEM & SANDBOXING

### 9.1 Architecture

- **Runtime**: WASM (WebAssembly) with Wasmtime
- **Language**: Plugins can be written in any language that compiles to WASM (Rust, Go, C, AssemblyScript, TinyGo)
- **SDK**: Rust crate `promptos-sdk` with typed bindings for the plugin API

### 9.2 Plugin Capabilities

```rust
// Plugin API — capabilities exposed to plugins

// Read-only prompt access (compiler input)
fn get_prompt_text() -> String;
fn get_prompt_ast() -> Option<JsonValue>;  // Serialized AST
fn get_target_model() -> String;
fn get_mode() -> String;

// Diagnostic reporting
fn report_diagnostic(severity: String, message: String, span: Option<Span>);

// Optimization pass registration
fn register_pass(name: String, version: String, priority: u8);

// Storage
fn get_storage(key: String) -> Option<String>;  // Plugin-scoped KV store
fn set_storage(key: String, value: String);
```

### 9.3 Plugin Security Model

| Capability | Default | Grantable |
|---|---|---|
| Read input prompt | ✓ Always | — |
| Read AST | ✓ Always | — |
| Read target model / mode | ✓ Always | — |
| Report diagnostics | ✓ Always | — |
| Plugin-scoped KV store | ✓ Always | — |
| Network access | ✗ Never | Via manifest permission |
| Filesystem access (outside plugin dir) | ✗ Never | Via manifest permission |
| Access API keys | ✗ Never | Via manifest permission (user must grant per-key) |
| Execute shell commands | ✗ Never | Never grantable |
| Access other plugin data | ✗ Never | Never grantable |

### 9.4 Plugin Manifest

```toml
[plugin]
name = "my-optimizer"
version = "1.0.0"
author = "PromptOS User"
description = "Custom optimization pass"

[permissions]
network = false
filesystem = false
api_keys = []  # Empty = no keys; ["openai"] = request access to OpenAI key
```

---

## 10. SECURITY ARCHITECTURE

### 10.1 Threat Model

| Threat | Mitigation |
|---|---|
| API key exfiltration via plugin | Plugin sandbox: no default access; explicit grant required; user confirmed per access |
| Compiled prompt interception | All API communication over TLS 1.3; certificate pinning for known providers |
| Local model file tampering | SHA-256 verification on every load |
| Update package tampering | Ed25519 signature verification in Sparkle framework |
| Unauthorized local file access | macOS Sandbox (Hardened Runtime); app container boundaries |
| Memory scraping for keys | Keychain-backed, locked memory during use, zeroing after use |
| Side-channel via timing attacks | Not a relevant threat for a desktop compilation tool (no multi-tenant environment) |

### 10.2 Code Signing & Notarization

```
Build → Sign with Developer ID Application certificate
  → Bundle into DMG
  → Submit to Apple Notary (notarytool)
  → Staple notarization ticket to DMG
  → Distribute
```

**Entitlements (Hardened Runtime)**:
```
com.apple.security.cs.allow-jit = YES (for WASM runtime)
com.apple.security.cs.allow-unsigned-executable-memory = YES (for WASM runtime)
com.apple.security.cs.disable-library-validation = YES (for llama.cpp dynamic loading)
com.apple.security.network.client = YES (for API calls, updates)
com.apple.security.files.user-selected.read-only = YES
```

### 10.3 Encryption

| Data | Encryption | Key Management |
|---|---|---|
| API keys | macOS Keychain (AES-256-GCM) | System-managed, device-bound |
| Telemetry database (opt-in) | SQLite + SQLCipher (AES-256-CBC) | Derived from device UID + app-specific salt |
| Plugin storage | Plugin-scoped file encryption (AES-256-GCM) | Per-plugin key derived from app-specific seed |
| Network transport | TLS 1.3 | System CA trust store + certificate pinning |

---

## 11. EVALUATION FRAMEWORK

### 11.1 Benchmark Suite

The evaluation framework is a critical component — it makes the "optimizer" claim falsifiable and measurable.

**Benchmark Categories**:

| Category | # Prompts | Example Task | Scoring Method |
|---|---|---|---|
| Code Generation | 50 | "Write a Python function that merges two sorted arrays" | Reference implementation comparison (BLEU, exact match, functional tests) |
| Writing & Analysis | 50 | "Analyze the causes of the French Revolution" | Rubric scoring (structure, depth, accuracy, citations) |
| Structured Extraction | 30 | "Extract all dates, amounts, and parties from this contract" | Exact field match, partial credit for semantic match |
| Classification & Reasoning | 30 | "Classify this email as spam/not-spam and explain" | Accuracy + reasoning quality (rubric) |
| Instruction Following | 40 | Multi-constraint prompts with 5-7 explicit constraints | Constraint satisfaction rate (pass/fail per constraint) |

### 11.2 A/B Harness

```rust
struct ABHarness {
    target_model: ModelId,
    evaluator: Box<dyn OutputEvaluator>,
}

impl ABHarness {
    fn run_ab_test(
        &self,
        raw_prompt: &str,
        compiled_prompt: &CompiledPrompt,
        api_key: &ApiKey,
    ) -> Result<ABTestResult, HarnessError>;
}

struct ABTestResult {
    raw_response: ModelResponse,
    compiled_response: ModelResponse,
    raw_score: f32,        // 0-10 quality score
    compiled_score: f32,   // 0-10 quality score
    improvement_pct: f32,  // ((compiled - raw) / raw) * 100
    raw_cost: f64,
    compiled_cost: f64,
    cost_savings_pct: f32,
    raw_latency_ms: u64,
    compiled_latency_ms: u64,
    determinism_score: f32, // How consistent are results across 3 runs
}
```

### 11.3 Regression Testing

- Every change to optimization passes must pass the evaluation suite without regressions
- CI pipeline runs the evaluation suite and compares scores to baseline
- A PR that degrades any category by >2% is blocked
- Model profile updates are validated against the evaluation suite before shipping

---

## 12. UPDATE SYSTEM (Sparkle)

### 12.1 Architecture

- **Framework**: Sparkle 2.x (objective-C, Swift-compatible)
- **Update feed**: Signed AppCast XML hosted at `https://update.promptos.app/appcast.xml`
- **Signature**: Ed25519 (private key held by PromptOS maintainers, public key bundled in app)
- **Update package**: Signed ZIP archive containing `.app` bundle
- **Delivery**: Direct download + delta updates (Sparkle's binary diff support)

### 12.2 Update Flow

```
1. App launches → Sparkle checks feed
2. Feed XML verified with Ed25519 public key
3. New version available → Notification shown
4. User accepts → Download update package
5. Package signature verified (Ed25519)
6. Package extracted → new .app staged
7. App relaunched with new version
8. Old version archived (rollback capability)
```

### 12.3 Rollback

- Previous version preserved in `/Applications/PromptOS (Previous).app` for one update cycle
- If new version crashes within first 3 launches, offer rollback
- Rollback is user-initiated via menu: "Revert to Previous Version"

---

## 13. CI/CD & RELEASE ENGINEERING

### 13.1 Build Pipeline

```
┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
│   lint    │──▶  test    │──▶  build    │──▶  package  │
│  (clippy) │   │(unit+int)│   │(release) │   │(DMG)     │
└──────────┘   └──────────┘   └──────────┘   └──────────┘
                                                  │
                                                  ▼
                                          ┌──────────┐
                                          │  sign +   │
                                          │ notarize  │
                                          └──────────┘
                                                  │
                                                  ▼
                                          ┌──────────┐
                                          │  publish  │
                                          │ (GitHub)  │
                                          └──────────┘
```

### 13.2 CI Steps

1. **Lint**: `cargo clippy` (Rust), SwiftLint (Swift)
2. **Test**: `cargo test` (Rust unit + integration), XCTest (Swift UI tests)
3. **Evaluation**: Run benchmark suite, compare scores to baseline, fail if regression >2%
4. **Build**: Release build for both architectures (arm64 + x86_64)
5. **Universal binary**: `lipo` to create universal binary for Intel + Apple Silicon
6. **Bundle**: macOS `.app` bundle with all resources (local model GGUF, profiles, Sparkle framework)
7. **Package**: DMG creation with `create-dmg` or custom script
8. **Sign**: `codesign --deep --force --verify-verbose --options runtime --timestamp --entitlements`
9. **Notarize**: `xcrun notarytool submit --apple-id --password --team-id`
10. **Staple**: `xcrun stapler staple`
11. **Verify**: `spctl --assess --verbose --type exec` + `codesign --verify --deep --strict`
12. **Upload**: GitHub Releases + update feed update

### 13.3 Release Types

| Type | Frequency | Qualifiers |
|---|---|---|
| Nightly | Daily | `main` branch, unsigned, dev-only |
| Beta | Weekly | Feature-complete, signed + notarized, opt-in testers |
| Release Candidate | Per-release | Full evaluation suite pass, signed + notarized |
| Stable | Monthly | RC that passes all criteria |

---

## 14. REPOSITORY STRUCTURE

```
promptos/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                  # Lint, test, build
│   │   ├── evaluation.yml          # Benchmark suite
│   │   └── release.yml             # Build, sign, notarize, publish
│   └── ISSUE_TEMPLATE/
├── crates/
│   ├── promptos-core/              # Compiler core (AST, passes, codegen)
│   │   ├── src/
│   │   │   ├── lexer.rs
│   │   │   ├── parser.rs
│   │   │   ├── ast.rs
│   │   │   ├── passes/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── redundancy.rs
│   │   │   │   ├── ambiguity.rs
│   │   │   │   ├── context_optimizer.rs
│   │   │   │   ├── instruction_strength.rs
│   │   │   │   ├── format_normalizer.rs
│   │   │   │   ├── token_budget.rs
│   │   │   │   ├── prioritization.rs
│   │   │   │   ├── cot_scaffolding.rs
│   │   │   │   ├── persona.rs
│   │   │   │   └── few_shot.rs
│   │   │   ├── codegen.rs
│   │   │   ├── verification.rs
│   │   │   └── diagnostics.rs
│   │   └── Cargo.toml
│   ├── promptos-llama/             # llama.cpp FFI bridge
│   │   ├── src/
│   │   │   ├── bridge.rs
│   │   │   ├── model.rs
│   │   │   ├── inference.rs
│   │   │   └── fallback.rs
│   │   ├── llama.cpp/              # Submodule or vendored
│   │   └── Cargo.toml
│   ├── promptos-profiles/          # Model profiles & registry
│   │   ├── src/
│   │   ├── profiles/               # Bundled TOML profiles
│   │   └── Cargo.toml
│   ├── promptos-history/           # Prompt history storage
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── promptos-plugin/            # Plugin runtime (WASM)
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── promptos-provider/          # Provider abstraction layer
│   │   ├── src/
│   │   │   ├── traits.rs
│   │   │   ├── anthropic.rs
│   │   │   ├── openai.rs
│   │   │   └── google.rs
│   │   └── Cargo.toml
│   ├── promptos-eval/              # Evaluation framework
│   │   ├── src/
│   │   ├── benchmarks/            # Benchmark prompt dataset
│   │   └── Cargo.toml
│   └── promptos-keychain/          # macOS Keychain integration
│       ├── src/
│       └── Cargo.toml
├── swift/
│   ├── PromptOSApp/
│   │   ├── PromptOSApp.swift
│   │   ├── Views/
│   │   ├── ViewModels/
│   │   ├── Services/
│   │   └── Resources/
│   └── PromptOSApp.xcodeproj
├── plugins/                        # Example plugins
│   └── example-plugin/
├── docs/
│   ├── architecture.md
│   ├── compiler.md
│   ├── plugin-sdk.md
│   └── contributing.md
├── scripts/
│   ├── build.sh
│   ├── sign.sh
│   └── notarize.sh
├── Cargo.toml                      # Workspace
├── Makefile
└── README.md
```

---

## 15. CODING STANDARDS

| Language | Standard | Formatter | Linter | Notes |
|---|---|---|---|---|
| Rust | Rust style guide (rustfmt) | `rustfmt` | `clippy` (all warnings) | No unsafe blocks without review; `#![forbid(unsafe_code)]` in all crates except promptos-llama |
| Swift | Swift API Design Guidelines | `swift-format` | SwiftLint | MVVM pattern; SwiftUI views minimal logic; ViewModels contain state |
| C++ (llama.cpp) | Google C++ Style | `clang-format` | `clang-tidy` | Only for llama.cpp; no new C++ code outside llama_bridge |
| TOML/YAML | 2-space indent | — | — | — |
| General | Conventional commits | — | — | `feat:` `fix:` `chore:` `docs:` `test:` |

---

## 16. TESTING ARCHITECTURE

### 16.1 Test Levels

| Level | Scope | Tool | CI Requirement |
|---|---|---|---|
| Unit | Individual functions, passes | `cargo test` | Blocking |
| Integration | Compiler pipeline end-to-end | `cargo test --test integration` | Blocking |
| Evaluation | Benchmark suite vs. baseline | `cargo run --bin evaluate` | Blocking (regression check) |
| UI | SwiftUI view tests | XCTest | Warning (no GUI in CI) |
| Smoke | Full app: launch → compile → display | Manual / XCTest UI | Pre-release |

### 16.2 Test Organization

- Unit tests live alongside code in `#[cfg(test)] mod tests`
- Integration tests in `tests/` directory at crate level
- Evaluation harness in `promptos-eval` crate with its own benchmark dataset
- Test fixtures (sample prompts, expected ASTs, expected compiled outputs) in `tests/fixtures/`

---

## 17. PERFORMANCE BUDGET

### 17.1 Startup Budget

| Phase | Target (Apple Silicon) | Target (Intel) |
|---|---|---|
| App launch to UI ready | <500ms | <1s |
| Model load (first launch) | <5s | <8s |
| Model load (subsequent) | <2s | <4s |
| Total cold start to ready | <5s | <10s |

### 17.2 Compilation Budget

| Phase | Time Budget (4K token input) |
|---|---|
| Lexical analysis + parsing | <50ms |
| Semantic analysis (local model) | <1.5s |
| Optimization passes (all 10) | <100ms |
| Code generation | <10ms |
| Verification | <50ms |
| Diff computation | <100ms |
| Total (with local model) | <2s |
| Total (rule-based fallback) | <200ms |

### 17.3 Memory Budget

| Phase | Memory Budget |
|---|---|
| Idle (no model loaded) | <150 MB |
| Idle (model loaded, no compilation) | <500 MB |
| Active compilation (input + AST + passes) | <800 MB |
| Peak (model inference + compilation) | <1.5 GB |

---

## 18. LOGGING & DIAGNOSTICS

### 18.1 Log Levels

```
Error:   Compilation failures, model load failures, crash conditions
Warning: Fallback activation, slow performance, profile staleness
Info:    Compilation start/end, mode changes, model loads, updates
Debug:   Pass-level details, AST state dumps, timing breakdowns
Trace:   Full inference input/output (development only, not in release)
```

### 18.2 Log Storage

- **Location**: `~/Library/Logs/com.promptos.app/`
- **Format**: Structured JSON lines (one JSON object per line)
- **Retention**: Last 7 days, rotated daily, compressed (gzip after rotation)
- **Max size**: 100 MB total (old logs deleted beyond cap)

### 18.3 Crash Reporting

- **Framework**: Custom crash reporter (lightweight, privacy-first) or PLCrashReporter
- **Capture**: Mach exception handler + signal handler
- **Content**: Stack trace, register state, app version, OS version, hardware model, compilation state at crash
- **No content**: Prompts, API keys, model outputs, or any user data
- **Submission**: Opt-in dialog on next launch with clear description of what will be sent

---

## 19. DEPENDENCY MANAGEMENT

### 19.1 Rust Dependencies (Key)

| Crate | Purpose | License |
|---|---|---|
| `serde` / `serde_json` | Serialization | MIT/Apache 2.0 |
| `toml` | Profile parsing | MIT/Apache 2.0 |
| `rmp-serde` | MessagePack (history) | MIT/Apache 2.0 |
| `zstd` | Compression (history) | BSD-3-Clause |
| `tokio` | Async runtime | MIT |
| `reqwest` | HTTP client | MIT/Apache 2.0 |
| `wasmtime` | WASM runtime (plugins) | Apache 2.0 |
| `security-framework` | macOS Keychain | MIT/Apache 2.0 |
| `candle` or `llama-cpp-2` | llama.cpp bindings | MIT/Apache 2.0 |
| `clap` | CLI argument parsing | MIT/Apache 2.0 |
| `tracing` | Structured logging | MIT |
| `uuid` | History entry IDs | MIT/Apache 2.0 |

### 19.2 Swift Dependencies

| Framework | Purpose | License |
|---|---|---|
| Sparkle | Automatic updates | MIT |
| SwiftUI | UI framework | Apple EULA |

---

## 20. GOVERNANCE IMPLEMENTATION

### 20.1 Repository Administration
- **License**: Apache 2.0 with LLVM exception for compiler passes (optional)
- **CLA**: Apache ICLA for contributions >100 lines; DCO (Signed-off-by) for smaller contributions
- **Branch protection**: `main` branch requires 1 review for regular PRs, 2 for compiler-core PRs
- **CODEOWNERS**: `crates/promptos-core/` requires compiler-team review

### 20.2 RFC Process
1. Author drafts RFC in `docs/rfcs/` as PR
2. 7-day minimum review period
3. Maintainer votes: +1 (approve), -1 (reject), 0 (abstain)
4. Majority +1 required for approval; single -1 blocks for discussion
5. Approved RFCs merged; implementation tracked in project board

---

## 21. APPENDIX: ARCHITECTURE DECISIONS LOG

| ADR # | Decision | Rationale | Date |
|---|---|---|---|
| 001 | Rust for compiler core | Memory safety without GC, cross-platform, excellent FFI, performance, ecosystem (serde, wasmtime, tokio) | 2026-01-15 |
| 002 | SwiftUI for UI layer | Native macOS integration, performance, Swift concurrency, native accessibility support | 2026-01-15 |
| 003 | llama.cpp (GGUF) for local inference | Cross-platform, CPU+GPU, supports Intel Macs, mature GGUF ecosystem | 2026-01-15 |
| 004 | Q4_K_M quantization | Best quality/size tradeoff; fits in DMG budget | 2026-01-15 |
| 005 | Model bundled in DMG (not fetched) | Fully offline immediately; no network required for first use | 2026-01-15 |
| 006 | Apache 2.0 license | Patent protection for contributors; industry standard for infrastructure OSS | 2026-01-15 |
| 007 | Sparkle for updates | De facto standard for macOS OSS apps; signed updates; delta support | 2026-01-15 |
| 008 | WASM + Wasmtime for plugins | Memory-safe, sandboxed, language-agnostic, determinism, small runtime | 2026-01-15 |
| 009 | MessagePack for history storage | Compact binary format, faster than JSON, schema-less, good library support for Rust | 2026-01-15 |
| 010 | macOS v1 only with documented cross-platform path | Focus resources on quality v1; architectural decisions preserve cross-platform option | 2026-01-15 |

---

## 22. APPENDIX: CROSS-PLATFORM READINESS CHECKLIST

| Component | Current (v1, macOS) | v2 (Windows) | v2 (Linux) |
|---|---|---|---|
| Compiler core (Rust) | ✅ Unchanged | ✅ Unchanged | ✅ Unchanged |
| Provider abstraction | ✅ Unchanged | ✅ Unchanged | ✅ Unchanged |
| Evaluation harness | ✅ Unchanged | ✅ Unchanged | ✅ Unchanged |
| Model profiles | ✅ Unchanged | ✅ Unchanged | ✅ Unchanged |
| llama.cpp | ✅ Metal | ✅ CUDA/Vulkan | ✅ Vulkan |
| UI | SwiftUI | WinUI/Tauri | GTK/Tauri |
| Window management | AppKit | Win32 | X11/Wayland |
| Keychain | Security.framework | WinCredentialManager | Secret Service |
| File system | Foundation | Win32 | `$XDG_*` |
| Updates | Sparkle | WinSparkle | Flatpak/Snap |
| Notifications | UserNotifications | Windows Toast | D-Bus Notify |
| Accessibility | VoiceOver | Narrator | Orca |

The compiler core, provider abstraction, evaluation harness, and model profiles require zero changes for Windows/Linux ports. Only the UI and OS-integration layers need platform-specific reimplementation.
