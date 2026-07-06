use super::*;
use async_trait::async_trait;

pub struct TokenBudgetOptimizationPass;

#[async_trait]
impl OptimizationPass for TokenBudgetOptimizationPass {
    fn name(&self) -> &str {
        "token-budget-optimization"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn should_run(&self, mode: CompilationMode, _config: &UserConfig) -> bool {
        matches!(mode, CompilationMode::Economy | CompilationMode::Balanced | CompilationMode::DeepAnalysis | CompilationMode::MissionCritical)
    }

    async fn run(&self, ast: &mut PromptRoot, ctx: &PassContext) -> Result<PassResult, PassError> {
        let context_limit = ctx.model_profile.as_ref().map_or(128000, |p| p.context_limit_input) as usize;
        let safety_margin = (context_limit as f64 * 0.1) as usize;
        let effective_limit = context_limit - safety_margin;

        let current_text = self.ast_to_text(ast);
        let current_tokens = current_text.split_whitespace().count();
        let mut tokens_saved = 0isize;

        if current_tokens > effective_limit {
            let overage = current_tokens - effective_limit;
            let mut removed = 0;

            let candidates = self.identify_compressible_nodes(ast);
            for idx in candidates.into_iter().rev() {
                if removed >= overage {
                    break;
                }
                if let Some(child) = ast.children.get(idx) {
                    let text = format!("{:?}", child);
                    let node_tokens = text.split_whitespace().count();
                    removed += node_tokens;
                    tokens_saved += node_tokens as isize;
                }
                ast.children.remove(idx);
            }

            if tokens_saved == 0 {
                return Err(PassError::AnalysisFailed(
                    "Cannot fit prompt within context budget even after optimization".to_string(),
                ));
            }
        }

        Ok(PassResult {
            pass_name: self.name().to_string(),
            tokens_saved,
            applied: tokens_saved > 0,
            description: format!("Optimized token budget, saved {} tokens", tokens_saved),
        })
    }

    fn verify(&self, ast: &PromptRoot, _original: &PromptRoot) -> Result<(), VerificationFailure> {
        if ast.children.is_empty() {
            return Err(VerificationFailure {
                pass_name: self.name().to_string(),
                reason: "Token budget optimization removed all content".to_string(),
            });
        }
        Ok(())
    }
}

impl TokenBudgetOptimizationPass {
    fn ast_to_text(&self, ast: &PromptRoot) -> String {
        ast.children.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>().join(" ")
    }

    fn identify_compressible_nodes(&self, ast: &PromptRoot) -> Vec<usize> {
        let mut scored: Vec<(usize, f32)> = ast.children.iter().enumerate().map(|(i, child)| {
            let priority = match child {
                PromptNode::Instruction(_) => 1.0,
                PromptNode::Constraint(_) => 0.95,
                PromptNode::RoleSpec(_) => 0.9,
                PromptNode::FormatSpec(_) => 0.85,
                PromptNode::Context(ctx) => ctx.relevance_score,
                PromptNode::Example(_) => 0.6,
                PromptNode::MetaInstruction(_) => 0.3,
                PromptNode::Section(s) => if s.children.is_empty() { 0.1 } else { 0.4 },
                PromptNode::Block(_) => 0.3,
                PromptNode::Root(_) => 1.0,
            };
            (i, priority)
        }).collect();

        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.iter().map(|(i, _)| *i).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::ModelProfileData;
    use crate::ast::Position;

    #[tokio::test]
    async fn test_token_budget_within_limit() {
        let mut ast = PromptRoot::new(vec![
            PromptNode::Instruction(Instruction {
                verb: InstructionVerb::Write,
                object: "Short".to_string(),
                modifiers: Vec::new(),
                confidence: 0.9,
                span: crate::ast::SourceSpan::new(Position::new(1, 1), Position::new(1, 5)),
            }),
        ]);
        ast.annotations.token_count_original = 5;
        let pass = TokenBudgetOptimizationPass;
        let profile = ModelProfileData {
            model_id: "claude-3.5-sonnet".to_string(),
            provider: "anthropic".to_string(),
            version: "1.0.0".to_string(),
            context_limit_input: 200000,
            context_limit_output: 8192,
            max_output_tokens: 8192,
            pricing_input_per_mtok: 3.0,
            pricing_output_per_mtok: 15.0,
        };
        let ctx = PassContext {
            model_profile: Some(profile.clone()),
            annotations: Annotations::default(),
            config: UserConfig::default(),
            target_model: "claude-3.5-sonnet".to_string(),
        };
        let result = pass.run(&mut ast, &ctx).await.unwrap();
        assert!(!result.applied);
    }
}
