use super::*;
use async_trait::async_trait;
use log::{info, debug};
use std::collections::HashSet;

pub struct RedundancyEliminationPass;

fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let set_a: HashSet<&str> = a.split_whitespace().collect();
    let set_b: HashSet<&str> = b.split_whitespace().collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

#[async_trait]
impl OptimizationPass for RedundancyEliminationPass {
    fn name(&self) -> &str {
        "redundancy-elimination"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn should_run(&self, mode: CompilationMode, _config: &UserConfig) -> bool {
        matches!(mode, CompilationMode::Economy | CompilationMode::Balanced | CompilationMode::DeepAnalysis | CompilationMode::MissionCritical)
    }

    async fn run(&self, ast: &mut PromptRoot, _ctx: &PassContext) -> Result<PassResult, PassError> {
        let before_tokens = ast.children.iter().map(|c| format!("{:?}", c).split_whitespace().count()).sum::<usize>();
        info!("Pass [redundancy] — entering, children={}", ast.children.len());
        let mut tokens_saved: isize = 0;
        let mut removed = 0;

        let mut i = 0;
        while i < ast.children.len() {
            let mut j = i + 1;
            while j < ast.children.len() {
                let content_i = self.get_content(&ast.children[i]);
                let content_j = self.get_content(&ast.children[j]);

                if !content_i.is_empty() && !content_j.is_empty() {
                    let sim = jaccard_similarity(&content_i, &content_j);
                    if sim > 0.85 {
                        let tokens = content_j.split_whitespace().count() as isize;
                        tokens_saved += tokens;
                        removed += 1;
                        ast.children.remove(j);
                        continue;
                    }
                }
                j += 1;
            }
            i += 1;
        }

        info!("Pass [redundancy] — exit, removed={}, tokens_saved={}, children_after={}", removed, tokens_saved, ast.children.len());
        Ok(PassResult {
            pass_name: self.name().to_string(),
            tokens_saved,
            applied: removed > 0,
            description: format!("Removed {} redundant nodes, saved {} tokens", removed, tokens_saved),
        })
    }

    fn verify(&self, ast: &PromptRoot, original: &PromptRoot) -> Result<(), VerificationFailure> {
        let original_instructions: HashSet<String> = original
            .children
            .iter()
            .filter_map(|c| {
                if let PromptNode::Instruction(instr) = c {
                    Some(instr.object.clone())
                } else {
                    None
                }
            })
            .collect();

        let compiled_instructions: HashSet<String> = ast
            .children
            .iter()
            .filter_map(|c| {
                if let PromptNode::Instruction(instr) = c {
                    Some(instr.object.clone())
                } else {
                    None
                }
            })
            .collect();

        for instr in &original_instructions {
            if !compiled_instructions.contains(instr) {
                let found = compiled_instructions.iter().any(|c| jaccard_similarity(c, instr) > 0.8);
                if !found {
                    return Err(VerificationFailure {
                        pass_name: self.name().to_string(),
                        reason: format!("Instruction lost: {}", instr),
                    });
                }
            }
        }
        Ok(())
    }
}

impl RedundancyEliminationPass {
    fn get_content(&self, node: &PromptNode) -> String {
        match node {
            PromptNode::Instruction(i) => i.object.clone(),
            PromptNode::Context(c) => c.content.clone(),
            PromptNode::Constraint(c) => c.value.clone(),
            PromptNode::Block(b) => b.content.clone(),
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redundancy_no_change() {
        let ast = PromptRoot::new(vec![
            PromptNode::Instruction(Instruction {
                verb: InstructionVerb::Write,
                object: "Write a poem".to_string(),
                modifiers: Vec::new(),
                confidence: 0.9,
                span: crate::ast::SourceSpan::new(Position::new(1, 1), Position::new(1, 15)),
            }),
        ]);
        let mut ast_clone = ast.clone();
        let pass = RedundancyEliminationPass;
        let ctx = PassContext {
            model_profile: None,
            annotations: Annotations::default(),
            config: UserConfig::default(),
            target_model: "claude-3.5-sonnet".to_string(),
        };
        let result = pass.run(&mut ast_clone, &ctx).await.unwrap();
        assert!(!result.applied);
        assert_eq!(result.tokens_saved, 0);
    }

    #[tokio::test]
    async fn test_redundancy_duplicates() {
        let ast = PromptRoot::new(vec![
            PromptNode::Instruction(Instruction {
                verb: InstructionVerb::Write,
                object: "Write a poem about nature".to_string(),
                modifiers: Vec::new(),
                confidence: 0.9,
                span: crate::ast::SourceSpan::new(Position::new(1, 1), Position::new(1, 25)),
            }),
            PromptNode::Instruction(Instruction {
                verb: InstructionVerb::Write,
                object: "Write a poem about nature".to_string(),
                modifiers: Vec::new(),
                confidence: 0.9,
                span: crate::ast::SourceSpan::new(Position::new(2, 1), Position::new(2, 25)),
            }),
        ]);
        let mut ast_clone = ast.clone();
        let pass = RedundancyEliminationPass;
        let ctx = PassContext {
            model_profile: None,
            annotations: Annotations::default(),
            config: UserConfig::default(),
            target_model: "claude-3.5-sonnet".to_string(),
        };
        let result = pass.run(&mut ast_clone, &ctx).await.unwrap();
        assert!(result.applied);
        assert!(result.tokens_saved > 0);
    }
}
