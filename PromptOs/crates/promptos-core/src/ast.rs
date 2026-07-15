use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: Position,
    pub end: Position,
}

impl SourceSpan {
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PromptNode {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptRoot {
    pub children: Vec<PromptNode>,
    pub annotations: Annotations,
}

impl PromptRoot {
    pub fn new(children: Vec<PromptNode>) -> Self {
        Self {
            children,
            annotations: Annotations::default(),
        }
    }

    pub fn builder() -> PromptRootBuilder {
        PromptRootBuilder::new()
    }
}

pub struct PromptRootBuilder {
    children: Vec<PromptNode>,
}

impl PromptRootBuilder {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn add(mut self, node: PromptNode) -> Self {
        self.children.push(node);
        self
    }

    pub fn build(self) -> PromptRoot {
        PromptRoot::new(self.children)
    }
}

impl Default for PromptRootBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub heading: String,
    pub level: u8,
    pub children: Vec<PromptNode>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlockType {
    Paragraph,
    List,
    Code,
    Quote,
    Table,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub block_type: BlockType,
    pub content: String,
    pub children: Vec<PromptNode>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InstructionVerb {
    Generate,
    Analyze,
    Extract,
    Classify,
    Summarize,
    Write,
    Explain,
    Compare,
    Translate,
    Rewrite,
    Expand,
    Reduce,
    Format,
    Convert,
    Search,
    Calculate,
    Debug,
    Optimize,
    Design,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Modifier {
    Adjective(String),
    Adverb(String),
    Scope(String),
    Quality(String),
    Tone(String),
    Audience(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    pub verb: InstructionVerb,
    pub object: String,
    pub modifiers: Vec<Modifier>,
    pub confidence: f32,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ContextType {
    Background,
    Definition,
    Reference,
    Data,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub content: String,
    pub context_type: ContextType,
    pub relevance_score: f32,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConstraintType {
    Positive,
    Negative,
    Length,
    Format,
    Content,
    Time,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConstraintSeverity {
    Required,
    Preferred,
    Suggested,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub constraint_type: ConstraintType,
    pub value: String,
    pub severity: ConstraintSeverity,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormatSpec {
    pub format_type: String,
    pub detail: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleSpec {
    pub role: String,
    pub traits: Vec<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Example {
    pub input: String,
    pub output: String,
    pub label: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaInstruction {
    pub content: String,
    pub meta_type: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OutputType {
    Text,
    Code,
    Json,
    Structured,
    ImageDesc,
    Table,
    List,
    Markdown,
    Html,
    Csv,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GapType {
    MissingInstruction,
    MissingContext,
    MissingFormat,
    MissingRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextGap {
    pub gap_type: GapType,
    pub description: String,
    pub suggested_addition: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PassStatus {
    Applied,
    Skipped,
    Failed,
    Reverted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizationEntry {
    pub pass_name: String,
    pub pass_version: String,
    pub status: PassStatus,
    pub tokens_saved: isize,
    pub description: String,
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Suggestion,
    Info,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub span: Option<SourceSpan>,
    pub diagnostic_code: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intent {
    pub primary_task: String,
    pub domain: Option<String>,
    pub output_type: OutputType,
    pub complexity: Complexity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ambiguity {
    pub text: String,
    pub span: SourceSpan,
    pub interpretations: Vec<String>,
    pub recommended_resolution: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contradiction {
    pub constraint_a: SourceSpan,
    pub constraint_b: SourceSpan,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    pub from: String,
    pub to: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotations {
    pub intent: Option<Intent>,
    pub detected_ambiguities: Vec<Ambiguity>,
    pub detected_contradictions: Vec<Contradiction>,
    pub detected_context_gaps: Vec<ContextGap>,
    pub dependencies: Vec<Dependency>,
    pub optimization_log: Vec<OptimizationEntry>,
    pub token_count_original: usize,
    pub token_count_compiled: usize,
    pub estimated_cost_original: f64,
    pub estimated_cost_compiled: f64,
    pub estimated_latency_original: f64,
    pub estimated_latency_compiled: f64,
    pub quality_score_original: f32,
    pub quality_score_compiled: f32,
    pub hallucination_risk: f32,
    pub diagnostics: Vec<Diagnostic>,
    pub compiler_version: String,
    pub timestamp: u64,
}

impl Default for Annotations {
    fn default() -> Self {
        Self {
            intent: None,
            detected_ambiguities: Vec::new(),
            detected_contradictions: Vec::new(),
            detected_context_gaps: Vec::new(),
            dependencies: Vec::new(),
            optimization_log: Vec::new(),
            token_count_original: 0,
            token_count_compiled: 0,
            estimated_cost_original: 0.0,
            estimated_cost_compiled: 0.0,
            estimated_latency_original: 0.0,
            estimated_latency_compiled: 0.0,
            quality_score_original: 0.0,
            quality_score_compiled: 0.0,
            hallucination_risk: 0.0,
            diagnostics: Vec::new(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompilationMode {
    Economy,
    Balanced,
    DeepAnalysis,
    MissionCritical,
    Benchmark,
}

impl CompilationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Economy => "economy",
            Self::Balanced => "balanced",
            Self::DeepAnalysis => "deep-analysis",
            Self::MissionCritical => "mission-critical",
            Self::Benchmark => "benchmark",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledPrompt {
    pub text: String,
    pub model_id: String,
    pub mode: CompilationMode,
    pub max_output_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub structured_output_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompilationMetrics {
    pub token_count_original: u32,
    pub token_count_compiled: u32,
    pub estimated_cost: f64,
    pub estimated_latency_ms: u64,
    pub quality_score: f32,
    pub hallucination_risk: f32,
    pub passes_applied: Vec<String>,
    pub compilation_time_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diff {
    pub operations: Vec<DiffOp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiffOp {
    Equal { text: String },
    Insert { text: String },
    Delete { text: String },
    Replace { old: String, new: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserConfig {
    pub default_model: Option<String>,
    pub default_mode: CompilationMode,
    pub optimization_aggressiveness: OptimizationAggressiveness,
    pub auto_apply_safe: bool,
    pub enabled_passes: Option<Vec<String>>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            default_model: None,
            default_mode: CompilationMode::Balanced,
            optimization_aggressiveness: OptimizationAggressiveness::Standard,
            auto_apply_safe: true,
            enabled_passes: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OptimizationAggressiveness {
    Conservative,
    Standard,
    Aggressive,
}
