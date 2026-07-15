use crate::ast::*;
use crate::passes::*;
use crate::semantic::PassContext;

pub struct PassManager {
    passes: Vec<Box<dyn OptimizationPass>>,
    mode: CompilationMode,
    #[allow(dead_code)]
    target_model: String,
    user_config: UserConfig,
}

impl PassManager {
    pub fn new(mode: CompilationMode, target_model: String, user_config: UserConfig) -> Self {
        Self {
            passes: Vec::new(),
            mode,
            target_model,
            user_config,
        }
    }

    pub fn register(&mut self, pass: Box<dyn OptimizationPass>) {
        self.passes.push(pass);
    }

    pub fn register_defaults(&mut self) {
        self.register(Box::new(RedundancyEliminationPass));
        self.register(Box::new(AmbiguityResolutionPass));
        self.register(Box::new(ContextWindowOptimizationPass));
        self.register(Box::new(InstructionStrengtheningPass));
        self.register(Box::new(FormatNormalizationPass));
        self.register(Box::new(TokenBudgetOptimizationPass));
        self.register(Box::new(PrioritizationOrderingPass));
        self.register(Box::new(CotScaffoldingPass));
        self.register(Box::new(PersonaReinforcementPass));
        self.register(Box::new(FewShotOptimizationPass));
    }

    pub fn passes(&self) -> &[Box<dyn OptimizationPass>] {
        &self.passes
    }

    pub async fn run_all(&mut self, ast: &mut PromptRoot, ctx: &PassContext) -> Vec<PassResult> {
        let mut results = Vec::new();
        for pass in &self.passes {
            if pass.should_run(self.mode, &self.user_config) {
                let original = ast.clone();
                match pass.run(ast, ctx).await {
                    Ok(result) => {
                        if result.applied {
                            match pass.verify(ast, &original) {
                                Ok(()) => {
                                    results.push(result);
                                }
                                Err(_) => {
                                    *ast = original;
                                    results.push(PassResult {
                                        pass_name: pass.name().to_string(),
                                        tokens_saved: 0,
                                        applied: false,
                                        description: format!(
                                            "{}: reverted - verification failed",
                                            pass.name()
                                        ),
                                    });
                                }
                            }
                        } else {
                            results.push(result);
                        }
                    }
                    Err(e) => {
                        results.push(PassResult {
                            pass_name: pass.name().to_string(),
                            tokens_saved: 0,
                            applied: false,
                            description: format!("{}: failed - {:?}", pass.name(), e),
                        });
                    }
                }
            } else {
                results.push(PassResult {
                    pass_name: pass.name().to_string(),
                    tokens_saved: 0,
                    applied: false,
                    description: format!("{}: skipped", pass.name()),
                });
            }
        }
        results
    }

    pub async fn run_selected(
        &mut self,
        ast: &mut PromptRoot,
        ctx: &PassContext,
        pass_names: &[&str],
    ) -> Vec<PassResult> {
        let mut results = Vec::new();
        for pass in &self.passes {
            if pass_names.contains(&pass.name()) {
                let original = ast.clone();
                match pass.run(ast, ctx).await {
                    Ok(result) => {
                        if result.applied {
                            match pass.verify(ast, &original) {
                                Ok(()) => results.push(result),
                                Err(_) => {
                                    *ast = original;
                                    results.push(PassResult {
                                        pass_name: pass.name().to_string(),
                                        tokens_saved: 0,
                                        applied: false,
                                        description: format!(
                                            "{}: reverted - verification failed",
                                            pass.name()
                                        ),
                                    });
                                }
                            }
                        } else {
                            results.push(result);
                        }
                    }
                    Err(e) => {
                        results.push(PassResult {
                            pass_name: pass.name().to_string(),
                            tokens_saved: 0,
                            applied: false,
                            description: format!("{}: failed - {:?}", pass.name(), e),
                        });
                    }
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pass_manager_empty() {
        let mut pm = PassManager::new(
            CompilationMode::Balanced,
            "claude-3.5-sonnet".to_string(),
            UserConfig::default(),
        );
        let mut root = PromptRoot::new(vec![]);
        let ctx = PassContext {
            model_profile: None,
            annotations: Annotations::default(),
            config: UserConfig::default(),
            target_model: "claude-3.5-sonnet".to_string(),
        };
        let results = pm.run_all(&mut root, &ctx).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_pass_manager_with_passes() {
        let mut pm = PassManager::new(
            CompilationMode::Balanced,
            "claude-3.5-sonnet".to_string(),
            UserConfig::default(),
        );
        pm.register_defaults();
        let mut root = PromptRoot::new(vec![]);
        let ctx = PassContext {
            model_profile: None,
            annotations: Annotations::default(),
            config: UserConfig::default(),
            target_model: "claude-3.5-sonnet".to_string(),
        };
        let results = pm.run_all(&mut root, &ctx).await;
        assert!(!results.is_empty());
    }
}
