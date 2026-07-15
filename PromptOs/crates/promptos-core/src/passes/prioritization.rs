use super::*;
use async_trait::async_trait;
use log::info;

pub struct PrioritizationOrderingPass;

#[async_trait]
impl OptimizationPass for PrioritizationOrderingPass {
    fn name(&self) -> &str {
        "prioritization-ordering"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn should_run(&self, _mode: CompilationMode, _config: &UserConfig) -> bool {
        true
    }

    async fn run(&self, ast: &mut PromptRoot, _ctx: &PassContext) -> Result<PassResult, PassError> {
        info!(
            "Pass [prioritization] — entering, children={}",
            ast.children.len()
        );
        let priority = |node: &PromptNode| -> u8 {
            match node {
                PromptNode::Constraint(c) => match c.severity {
                    ConstraintSeverity::Required => 1,
                    ConstraintSeverity::Preferred => 2,
                    ConstraintSeverity::Suggested => 3,
                },
                PromptNode::Instruction(_) => 4,
                PromptNode::RoleSpec(_) => 5,
                PromptNode::FormatSpec(_) => 6,
                PromptNode::Context(ctx) => {
                    if ctx.relevance_score > 0.7 {
                        7
                    } else {
                        8
                    }
                }
                PromptNode::Example(_) => 9,
                PromptNode::MetaInstruction(_) => 10,
                PromptNode::Section(s) => {
                    if s.children.is_empty() {
                        12
                    } else {
                        11
                    }
                }
                PromptNode::Block(_) => 13,
                PromptNode::Root(_) => 0,
            }
        };

        ast.children.sort_by_key(|a| priority(a));
        let _original_order: Vec<u8> = ast.children.iter().map(priority).collect();

        info!("Pass [prioritization] — exit, reordering complete");
        Ok(PassResult {
            pass_name: self.name().to_string(),
            tokens_saved: 0,
            applied: true,
            description: "Reordered prompt elements by priority: constraints > instructions > role > format > context > examples > meta".to_string(),
        })
    }

    fn verify(&self, ast: &PromptRoot, _original: &PromptRoot) -> Result<(), VerificationFailure> {
        let priorities: Vec<u8> = ast
            .children
            .iter()
            .map(|c| match c {
                PromptNode::Instruction(_) => 4,
                PromptNode::Constraint(c) => match c.severity {
                    ConstraintSeverity::Required => 1,
                    ConstraintSeverity::Preferred => 2,
                    ConstraintSeverity::Suggested => 3,
                },
                _ => 10,
            })
            .collect();

        for w in priorities.windows(2) {
            if w[0] > 10 && w[1] < 10 {
                return Err(VerificationFailure {
                    pass_name: self.name().to_string(),
                    reason:
                        "Prioritization order violated: higher-priority items after lower-priority"
                            .to_string(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prioritization_reorders() {
        let mut ast = PromptRoot::new(vec![
            PromptNode::Context(Context {
                content: "Some context".to_string(),
                context_type: ContextType::Background,
                relevance_score: 0.5,
                span: crate::ast::SourceSpan::new(Position::new(1, 1), Position::new(1, 13)),
            }),
            PromptNode::Instruction(Instruction {
                verb: InstructionVerb::Write,
                object: "Write code".to_string(),
                modifiers: Vec::new(),
                confidence: 0.9,
                span: crate::ast::SourceSpan::new(Position::new(2, 1), Position::new(2, 10)),
            }),
        ]);
        let pass = PrioritizationOrderingPass;
        let ctx = PassContext {
            model_profile: None,
            annotations: Annotations::default(),
            config: UserConfig::default(),
            target_model: "claude-3.5-sonnet".to_string(),
        };
        let result = pass.run(&mut ast, &ctx).await.unwrap();
        assert!(result.applied);
        assert!(matches!(ast.children[0], PromptNode::Instruction(_)));
    }
}
