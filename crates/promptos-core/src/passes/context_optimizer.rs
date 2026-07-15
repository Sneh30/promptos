use super::*;
use async_trait::async_trait;
use log::info;

pub struct ContextWindowOptimizationPass;

#[async_trait]
impl OptimizationPass for ContextWindowOptimizationPass {
    fn name(&self) -> &str {
        "context-window-optimization"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn should_run(&self, _mode: CompilationMode, _config: &UserConfig) -> bool {
        true
    }

    async fn run(&self, ast: &mut PromptRoot, ctx: &PassContext) -> Result<PassResult, PassError> {
        let context_limit = ctx.model_profile.as_ref().map_or(128000, |p| p.context_limit_input) as usize;
        let current_tokens = ast.annotations.token_count_original;
        info!("Pass [context_optimizer] — entering, tokens={}, context_limit={}", current_tokens, context_limit);
        let mut tokens_saved = 0isize;

        if current_tokens > context_limit / 2 {
            let mut scored: Vec<(usize, f32)> = ast.children.iter().enumerate().map(|(i, child)| {
                let score = self.score_relevance(child);
                (i, score)
            }).collect();

            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let keep_count = std::cmp::max(1, (scored.len() as f32 * 0.8) as usize);
            let to_remove: Vec<usize> = scored.iter().skip(keep_count).map(|(i, _)| *i).collect();

            for idx in to_remove.into_iter().rev() {
                if let Some(child) = ast.children.get(idx) {
                    let text = format!("{:?}", child);
                    tokens_saved += text.split_whitespace().count() as isize;
                }
                ast.children.remove(idx);
            }
        }

        info!("Pass [context_optimizer] — exit, tokens_saved={}, children_after={}", tokens_saved, ast.children.len());
        Ok(PassResult {
            pass_name: self.name().to_string(),
            tokens_saved,
            applied: tokens_saved > 0,
            description: format!("Optimized context window, saved {} tokens", tokens_saved),
        })
    }

    fn verify(&self, ast: &PromptRoot, _original: &PromptRoot) -> Result<(), VerificationFailure> {
        let has_instruction = ast.children.iter().any(|c| matches!(c, PromptNode::Instruction(_)));
        if !has_instruction && !ast.children.is_empty() {
            return Err(VerificationFailure {
                pass_name: self.name().to_string(),
                reason: "All instructions removed by context optimization".to_string(),
            });
        }
        Ok(())
    }
}

impl ContextWindowOptimizationPass {
    fn score_relevance(&self, node: &PromptNode) -> f32 {
        match node {
            PromptNode::Instruction(_) => 1.0,
            PromptNode::Constraint(_) => 0.95,
            PromptNode::RoleSpec(_) => 0.9,
            PromptNode::FormatSpec(_) => 0.85,
            PromptNode::Example(_) => 0.7,
            PromptNode::Context(ctx) => ctx.relevance_score,
            PromptNode::MetaInstruction(_) => 0.3,
            PromptNode::Section(s) => {
                if s.children.is_empty() { 0.2 } else { 0.5 }
            }
            PromptNode::Block(_) => 0.4,
            PromptNode::Root(_) => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_optimization_short_prompt() {
        let mut ast = PromptRoot::new(vec![
            PromptNode::Instruction(Instruction {
                verb: InstructionVerb::Write,
                object: "Write a short poem".to_string(),
                modifiers: Vec::new(),
                confidence: 0.9,
                span: crate::ast::SourceSpan::new(Position::new(1, 1), Position::new(1, 20)),
            }),
        ]);
        ast.annotations.token_count_original = 5;
        let pass = ContextWindowOptimizationPass;
        let ctx = PassContext {
            model_profile: None,
            annotations: Annotations::default(),
            config: UserConfig::default(),
            target_model: "claude-3.5-sonnet".to_string(),
        };
        let result = pass.run(&mut ast, &ctx).await.unwrap();
        assert!(!result.applied);
    }
}
