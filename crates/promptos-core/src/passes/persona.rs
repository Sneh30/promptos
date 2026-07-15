use super::*;
use async_trait::async_trait;
use log::info;

pub struct PersonaReinforcementPass;

#[async_trait]
impl OptimizationPass for PersonaReinforcementPass {
    fn name(&self) -> &str {
        "persona-reinforcement"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn should_run(&self, _mode: CompilationMode, _config: &UserConfig) -> bool {
        true
    }

    async fn run(&self, ast: &mut PromptRoot, ctx: &PassContext) -> Result<PassResult, PassError> {
        info!("Pass [persona] — entering, target_model={}", ctx.target_model);
        let mut persona_count = 0;

        for child in &ast.children {
            if let PromptNode::RoleSpec(role) = child {
                let reinforced = if ctx.target_model.contains("claude") {
                    format!("<role>\n{}\n</role>", role.role)
                } else if ctx.target_model.contains("gpt") || ctx.target_model.contains("openai") {
                    format!("System: You are {}.", role.role.trim_start_matches("You are").trim())
                } else {
                    role.role.clone()
                };

                if reinforced != role.role {
                    persona_count += 1;
                }
            }
        }

        info!("Pass [persona] — exit, reinforced={}", persona_count);
        Ok(PassResult {
            pass_name: self.name().to_string(),
            tokens_saved: 0,
            applied: persona_count > 0,
            description: format!("Reinforced {} persona specifications for model {}", 
                persona_count, ctx.target_model),
        })
    }

    fn verify(&self, _ast: &PromptRoot, _original: &PromptRoot) -> Result<(), VerificationFailure> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_persona_reinforcement_no_persona() {
        let mut ast = PromptRoot::new(vec![
            PromptNode::Instruction(Instruction {
                verb: InstructionVerb::Write,
                object: "Write code".to_string(),
                modifiers: Vec::new(),
                confidence: 0.9,
                span: crate::ast::SourceSpan::new(Position::new(1, 1), Position::new(1, 10)),
            }),
        ]);
        let pass = PersonaReinforcementPass;
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
