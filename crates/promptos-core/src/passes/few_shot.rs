use super::*;
use async_trait::async_trait;
use std::collections::HashSet;

pub struct FewShotOptimizationPass;

fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let set_a: HashSet<&str> = a.split_whitespace().collect();
    let set_b: HashSet<&str> = b.split_whitespace().collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 { 0.0 } else { intersection as f64 / union as f64 }
}

#[async_trait]
impl OptimizationPass for FewShotOptimizationPass {
    fn name(&self) -> &str {
        "few-shot-optimization"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn should_run(&self, _mode: CompilationMode, _config: &UserConfig) -> bool {
        true
    }

    async fn run(&self, ast: &mut PromptRoot, _ctx: &PassContext) -> Result<PassResult, PassError> {
        let mut optimized = 0;
        let mut tokens_saved = 0isize;
        let mut examples: Vec<(usize, Example)> = Vec::new();

        for (i, child) in ast.children.iter().enumerate() {
            if let PromptNode::Example(ex) = child {
                examples.push((i, ex.clone()));
            }
        }

        if examples.len() < 2 {
            return Ok(PassResult {
                pass_name: self.name().to_string(),
                tokens_saved: 0,
                applied: false,
                description: format!("{} examples found, need at least 2 for optimization", examples.len()),
            });
        }

        // Deduplicate similar examples
        let mut to_remove: Vec<usize> = Vec::new();
        for (i, (_, ex_a)) in examples.iter().enumerate() {
            for (j, (idx_b, ex_b)) in examples.iter().enumerate() {
                if i < j {
                    let sim = jaccard_similarity(&ex_a.input, &ex_b.input);
                    if sim > 0.85 {
                        to_remove.push(*idx_b);
                    }
                }
            }
        }

        to_remove.sort();
        to_remove.dedup();
        for idx in to_remove.into_iter().rev() {
            if let Some(child) = ast.children.get(idx) {
                let text = format!("{:?}", child);
                tokens_saved += text.split_whitespace().count() as isize;
            }
            ast.children.remove(idx);
            optimized += 1;
        }

        Ok(PassResult {
            pass_name: self.name().to_string(),
            tokens_saved,
            applied: optimized > 0,
            description: format!("Optimized {} few-shot examples, saved {} tokens", optimized, tokens_saved),
        })
    }

    fn verify(&self, ast: &PromptRoot, _original: &PromptRoot) -> Result<(), VerificationFailure> {
        let example_count = ast.children.iter().filter(|c| matches!(c, PromptNode::Example(_))).count();
        if example_count == 1 {
            return Err(VerificationFailure {
                pass_name: self.name().to_string(),
                reason: "Only 1 example remaining after optimization; at least 2 recommended".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_few_shot_no_examples() {
        let mut ast = PromptRoot::new(vec![
            PromptNode::Instruction(Instruction {
                verb: InstructionVerb::Write,
                object: "Write code".to_string(),
                modifiers: Vec::new(),
                confidence: 0.9,
                span: crate::ast::SourceSpan::new(Position::new(1, 1), Position::new(1, 10)),
            }),
        ]);
        let pass = FewShotOptimizationPass;
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
