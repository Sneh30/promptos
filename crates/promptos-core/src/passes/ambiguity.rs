use super::*;
use async_trait::async_trait;
use log::info;

pub struct AmbiguityResolutionPass;

#[async_trait]
impl OptimizationPass for AmbiguityResolutionPass {
    fn name(&self) -> &str {
        "ambiguity-resolution"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn should_run(&self, _mode: CompilationMode, _config: &UserConfig) -> bool {
        true
    }

    async fn run(&self, ast: &mut PromptRoot, _ctx: &PassContext) -> Result<PassResult, PassError> {
        let ambiguities = ast.annotations.detected_ambiguities.clone();
        info!("Pass [ambiguity] — entering, ambiguities={}", ambiguities.len());
        let mut resolved = 0usize;

        for ambiguity in &ambiguities {
            if ambiguity.confidence > 0.8 {
                if let Some(resolution) = &ambiguity.recommended_resolution {
                    for child in &mut ast.children {
                        self.resolve_in_node(child, &ambiguity.text, resolution);
                    }
                    resolved += 1;
                }
            }
        }

        info!("Pass [ambiguity] — exit, resolved={}, remaining={}", resolved, ambiguities.len() - resolved);
        Ok(PassResult {
            pass_name: self.name().to_string(),
            tokens_saved: 0,
            applied: resolved > 0,
            description: format!("Resolved {} ambiguities automatically, {} flagged for user", 
                resolved, ambiguities.len() - resolved),
        })
    }

    fn verify(&self, ast: &PromptRoot, _original: &PromptRoot) -> Result<(), VerificationFailure> {
        let unresolved: Vec<&Ambiguity> = ast.annotations.detected_ambiguities.iter()
            .filter(|a| a.confidence > 0.9 && a.recommended_resolution.is_some())
            .collect();
        if unresolved.len() > 5 {
            return Err(VerificationFailure {
                pass_name: self.name().to_string(),
                reason: format!("{} high-confidence ambiguities unresolved", unresolved.len()),
            });
        }
        Ok(())
    }
}

impl AmbiguityResolutionPass {
    fn resolve_in_node(&self, node: &mut PromptNode, target: &str, resolution: &str) {
        match node {
            PromptNode::Instruction(instr) => {
                if instr.object.contains(target) {
                    instr.object = instr.object.replace(target, resolution);
                }
            }
            PromptNode::Context(ctx) => {
                if ctx.content.contains(target) {
                    ctx.content = ctx.content.replace(target, resolution);
                }
            }
            PromptNode::Section(section) => {
                for child in &mut section.children {
                    self.resolve_in_node(child, target, resolution);
                }
            }
            PromptNode::Block(block) => {
                for child in &mut block.children {
                    self.resolve_in_node(child, target, resolution);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ambiguity_no_ambiguities() {
        let mut ast = PromptRoot::new(vec![]);
        let pass = AmbiguityResolutionPass;
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
