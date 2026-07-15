mod redundancy;
mod ambiguity;
mod context_optimizer;
mod instruction_strength;
mod format_normalizer;
mod token_budget;
mod prioritization;
mod cot_scaffolding;
mod persona;
mod few_shot;

pub use redundancy::*;
pub use ambiguity::*;
pub use context_optimizer::*;
pub use instruction_strength::*;
pub use format_normalizer::*;
pub use token_budget::*;
pub use prioritization::*;
pub use cot_scaffolding::*;
pub use persona::*;
pub use few_shot::*;

use crate::ast::*;
use crate::semantic::PassContext;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct PassResult {
    pub pass_name: String,
    pub tokens_saved: isize,
    pub applied: bool,
    pub description: String,
}

#[derive(Debug)]
pub enum PassError {
    AnalysisFailed(String),
    InvalidAST(String),
    VerificationFailed(String),
    Internal(String),
}

impl std::fmt::Display for PassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnalysisFailed(msg) => write!(f, "Analysis failed: {}", msg),
            Self::InvalidAST(msg) => write!(f, "Invalid AST: {}", msg),
            Self::VerificationFailed(msg) => write!(f, "Verification failed: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for PassError {}

#[derive(Debug)]
pub struct VerificationFailure {
    pub pass_name: String,
    pub reason: String,
}

#[async_trait]
pub trait OptimizationPass: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn should_run(&self, mode: CompilationMode, config: &UserConfig) -> bool;
    async fn run(&self, ast: &mut PromptRoot, ctx: &PassContext) -> Result<PassResult, PassError>;
    fn verify(&self, ast: &PromptRoot, original: &PromptRoot) -> Result<(), VerificationFailure>;
}
