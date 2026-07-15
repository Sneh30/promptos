use crate::ast::*;
use crate::codegen::{self, ModelCodeGenerator};
use crate::diagnostics::DiagnosticBuilder;
use crate::parser;
use crate::pass_manager::PassManager;
use crate::semantic::{ModelProfileData, PassContext, RuleBasedAnalyzer, SemanticAnalyzer};
use crate::verification;
use log::{debug, info};

pub struct Compiler {
    target_model: String,
    mode: CompilationMode,
    config: UserConfig,
    generator: Box<dyn ModelCodeGenerator>,
    pass_manager: PassManager,
}

impl Compiler {
    pub fn new(target_model: &str, mode: CompilationMode, config: UserConfig) -> Self {
        info!(
            "Compiler created — target_model={}, mode={:?}",
            target_model, mode
        );
        let generator = codegen::create_generator(target_model);
        let mut pass_manager = PassManager::new(mode, target_model.to_string(), config.clone());
        pass_manager.register_defaults();

        Self {
            target_model: target_model.to_string(),
            mode,
            config,
            generator,
            pass_manager,
        }
    }

    pub async fn compile(
        &mut self,
        input: &str,
        profile: Option<&ModelProfileData>,
    ) -> Result<CompilationResult, String> {
        let start = std::time::Instant::now();
        info!(
            "Compilation start — input_len={}, target_model={}, mode={:?}",
            input.len(),
            self.target_model,
            self.mode
        );

        let mut root = parser::parse(input)?;
        debug!("Parse complete — {} AST nodes", root.children.len());
        let original = root.clone();

        let analyzer = RuleBasedAnalyzer;
        let annotations = analyzer
            .analyze(&mut root)
            .await
            .map_err(|e| format!("Analysis error: {}", e))?;

        let mut diagnostics = DiagnosticBuilder::new();
        for diag in &annotations.diagnostics {
            match diag.severity {
                DiagnosticSeverity::Error => diagnostics.error(
                    &diag.diagnostic_code,
                    &diag.message,
                    diag.span,
                    diag.recommendation.as_deref(),
                ),
                DiagnosticSeverity::Warning => diagnostics.warning(
                    &diag.diagnostic_code,
                    &diag.message,
                    diag.span,
                    diag.recommendation.as_deref(),
                ),
                DiagnosticSeverity::Suggestion => diagnostics.suggestion(
                    &diag.diagnostic_code,
                    &diag.message,
                    diag.span,
                    diag.recommendation.as_deref(),
                ),
                DiagnosticSeverity::Info => diagnostics.info(
                    &diag.diagnostic_code,
                    &diag.message,
                    diag.span,
                    diag.recommendation.as_deref(),
                ),
            }
        }
        info!(
            "Analysis — {} diagnostics, {} errors, {} warnings",
            annotations.diagnostics.len(),
            annotations
                .diagnostics
                .iter()
                .filter(|d| matches!(d.severity, DiagnosticSeverity::Error))
                .count(),
            annotations
                .diagnostics
                .iter()
                .filter(|d| matches!(d.severity, DiagnosticSeverity::Warning))
                .count()
        );

        let ctx = PassContext {
            model_profile: profile.cloned(),
            annotations: root.annotations.clone(),
            config: self.config.clone(),
            target_model: self.target_model.clone(),
        };

        let pass_results = self.pass_manager.run_all(&mut root, &ctx).await;
        debug!(
            "Passes complete — {} applied, {} total",
            pass_results.iter().filter(|r| r.applied).count(),
            pass_results.len()
        );

        let context_limit = profile.map_or(128000, |p| p.context_limit_input);
        let verify_results =
            verification::verify_all(&root, &original, context_limit, &self.target_model);

        let compiled = self.generator.generate(&root, profile);
        let token_count_original = annotations.token_count_original;
        let token_count_compiled = compiled.text.split_whitespace().count();
        let compilation_time_ms = start.elapsed().as_millis() as u64;
        info!(
            "Codegen — output_len={}, tokens_original={}, tokens_compiled={}, time_ms={}",
            compiled.text.len(),
            token_count_original,
            token_count_compiled,
            compilation_time_ms
        );

        let tokens_saved = token_count_original as isize - token_count_compiled as isize;
        let cost_saved =
            tokens_saved as f64 * profile.map_or(0.0, |p| p.pricing_input_per_mtok) / 1_000_000.0;

        let result = CompilationResult {
            original_prompt: input.to_string(),
            compiled_text: compiled.text.clone(),
            metrics: CompilationMetrics {
                token_count_original: token_count_original as u32,
                token_count_compiled: token_count_compiled as u32,
                estimated_cost: cost_saved.max(0.0),
                estimated_latency_ms: compilation_time_ms,
                quality_score: root.annotations.quality_score_compiled,
                hallucination_risk: root.annotations.hallucination_risk,
                passes_applied: pass_results
                    .iter()
                    .filter(|r| r.applied)
                    .map(|r| r.pass_name.clone())
                    .collect(),
                compilation_time_ms,
            },
            diagnostics: diagnostics.build(),
            pass_results,
            verification_results: verify_results,
            diff: compute_diff(original, &root),
            target_model: self.target_model.clone(),
            mode: self.mode,
        };

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub original_prompt: String,
    pub compiled_text: String,
    pub metrics: CompilationMetrics,
    pub diagnostics: Vec<Diagnostic>,
    pub pass_results: Vec<crate::passes::PassResult>,
    pub verification_results: Vec<verification::VerificationResult>,
    pub diff: Diff,
    pub target_model: String,
    pub mode: CompilationMode,
}

fn compute_diff(_original: PromptRoot, _compiled: &PromptRoot) -> Diff {
    Diff {
        operations: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compile_empty_input() {
        let mut compiler = Compiler::new(
            "claude-3.5-sonnet",
            CompilationMode::Balanced,
            UserConfig::default(),
        );
        let result = compiler.compile("", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compile_simple_prompt() {
        let mut compiler = Compiler::new(
            "claude-3.5-sonnet",
            CompilationMode::Balanced,
            UserConfig::default(),
        );
        let result = compiler.compile("Write a poem about nature", None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.compiled_text.is_empty());
        assert!(result.metrics.token_count_original > 0);
    }
}
