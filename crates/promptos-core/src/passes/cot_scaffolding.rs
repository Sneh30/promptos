use super::*;
use async_trait::async_trait;
use log::info;

pub struct CotScaffoldingPass;

const COT_PHRASES: &[&str] = &[
    "Let's work through this step by step:",
    "Let me think through this carefully:",
    "I'll approach this systematically:",
    "Let's reason through this:",
];

#[async_trait]
impl OptimizationPass for CotScaffoldingPass {
    fn name(&self) -> &str {
        "cot-scaffolding"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn should_run(&self, mode: CompilationMode, _config: &UserConfig) -> bool {
        matches!(mode, CompilationMode::DeepAnalysis | CompilationMode::MissionCritical | CompilationMode::Balanced)
    }

    async fn run(&self, ast: &mut PromptRoot, ctx: &PassContext) -> Result<PassResult, PassError> {
        info!("Pass [cot_scaffolding] — entering, target_model={}", ctx.target_model);
        let needs_cot = ast.annotations.intent.as_ref().map_or(false, |i| {
            matches!(i.complexity, Complexity::Complex | Complexity::VeryComplex)
        });

        let model_supports_cot = self.model_supports_cot(&ctx.target_model);

        if needs_cot && model_supports_cot {
            let new_children = vec![
                PromptNode::MetaInstruction(MetaInstruction {
                    content: COT_PHRASES[0].to_string(),
                    meta_type: "reasoning-scaffold".to_string(),
                        span: crate::ast::SourceSpan::new(
                            Position::new(1, 1),
                            Position::new(1, COT_PHRASES[0].len() + 1),
                        ),
                }),
            ];
            let mut all = new_children;
            all.append(&mut ast.children);
            ast.children = all;
            info!("Pass [cot_scaffolding] — exit, applied=true, complexity=complex");

            Ok(PassResult {
                pass_name: self.name().to_string(),
                tokens_saved: -(COT_PHRASES[0].split_whitespace().count() as isize),
                applied: true,
                description: format!("Injected CoT scaffolding (model: {}, complexity requires reasoning)", ctx.target_model),
            })
        } else {
            info!("Pass [cot_scaffolding] — exit, applied=false, needs_cot={}", needs_cot);
            Ok(PassResult {
                pass_name: self.name().to_string(),
                tokens_saved: 0,
                applied: false,
                description: if !needs_cot {
                    "CoT not needed: task complexity is low".to_string()
                } else {
                    format!("CoT skipped: model {} may not benefit from CoT scaffolding", ctx.target_model)
                },
            })
        }
    }

    fn verify(&self, _ast: &PromptRoot, _original: &PromptRoot) -> Result<(), VerificationFailure> {
        Ok(())
    }
}

impl CotScaffoldingPass {

    fn model_supports_cot(&self, model_id: &str) -> bool {
        model_id.contains("claude") || model_id.contains("gpt") || model_id.contains("gemini")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cot_not_needed_for_simple() {
        let mut ast = PromptRoot::new(vec![]);
        ast.annotations.intent = Some(Intent {
            primary_task: "Write a sentence".to_string(),
            domain: None,
            output_type: OutputType::Text,
            complexity: Complexity::Simple,
        });
        let pass = CotScaffoldingPass;
        let ctx = PassContext {
            model_profile: None,
            annotations: ast.annotations.clone(),
             config: UserConfig::default(),
             target_model: "claude-3.5-sonnet".to_string(),
         };
         let result = pass.run(&mut ast, &ctx).await.unwrap();
         assert!(!result.applied);
     }

     #[tokio::test]
     async fn test_cot_applied_for_complex() {
         let mut ast = PromptRoot::new(vec![]);
         ast.annotations.intent = Some(Intent {
             primary_task: "Design a distributed system".to_string(),
             domain: None,
             output_type: OutputType::Text,
             complexity: Complexity::Complex,
         });
         let pass = CotScaffoldingPass;
         let ctx = PassContext {
             model_profile: None,
             annotations: ast.annotations.clone(),
            config: UserConfig::default(),
            target_model: "claude-3.5-sonnet".to_string(),
        };
        let result = pass.run(&mut ast, &ctx).await.unwrap();
        assert!(result.applied);
    }
}
