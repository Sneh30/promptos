use super::*;
use async_trait::async_trait;
use log::info;

pub struct InstructionStrengtheningPass;

const WEAK_PATTERNS: &[(&str, &str)] = &[
    ("could you", ""),
    ("maybe", ""),
    ("i'd like if", ""),
    ("if possible", ""),
    ("would you mind", ""),
    ("might you", ""),
    ("perhaps you could", ""),
    ("i was wondering if", ""),
    ("can you try to", ""),
];

const STRONG_PREFIXES: &[&str] = &[
    "You must", "Required:", "Critical:", "Important:",
    "Strictly", "Exactly", "Precisely",
];

#[async_trait]
impl OptimizationPass for InstructionStrengtheningPass {
    fn name(&self) -> &str {
        "instruction-strengthening"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn should_run(&self, mode: CompilationMode, _config: &UserConfig) -> bool {
        matches!(mode, CompilationMode::Balanced | CompilationMode::DeepAnalysis | CompilationMode::MissionCritical)
    }

    async fn run(&self, ast: &mut PromptRoot, _ctx: &PassContext) -> Result<PassResult, PassError> {
        info!("Pass [instruction_strength] — entering, children={}", ast.children.len());
        let mut strengthened = 0;
        let mut tokens_saved = 0isize;

        for child in &mut ast.children {
            if let PromptNode::Instruction(instr) = child {
                let original_len = instr.object.len();
                for (weak, replacement) in WEAK_PATTERNS {
                    let lower = instr.object.to_lowercase();
                    if lower.contains(weak) {
                        if replacement.is_empty() {
                            instr.object = instr.object.replace(
                                &instr.object[..weak.len() + 1],
                                "",
                            );
                        }
                        instr.object = instr.object.trim().to_string();
                        if !instr.object.starts_with("Write")
                            && !instr.object.starts_with("Analyze")
                            && !instr.object.starts_with("Explain")
                        {
                            // Strengthen by adding imperative prefix
                            instr.object = format!("{} {}", self.get_imperative(&instr.verb), instr.object);
                        }
                        strengthened += 1;
                        let new_len = instr.object.len();
                        tokens_saved += (original_len as isize - new_len as isize).max(0);
                        break;
                    }
                }
            }
        }

        info!("Pass [instruction_strength] — exit, strengthened={}, tokens_saved={}", strengthened, tokens_saved);
        Ok(PassResult {
            pass_name: self.name().to_string(),
            tokens_saved,
            applied: strengthened > 0,
            description: format!("Strengthened {} weak instructions", strengthened),
        })
    }

    fn verify(&self, ast: &PromptRoot, original: &PromptRoot) -> Result<(), VerificationFailure> {
        let orig_objects: Vec<&str> = original
            .children
            .iter()
            .filter_map(|c| {
                if let PromptNode::Instruction(i) = c {
                    Some(i.object.as_str())
                } else {
                    None
                }
            })
            .collect();

        let compiled_objects: Vec<&str> = ast
            .children
            .iter()
            .filter_map(|c| {
                if let PromptNode::Instruction(i) = c {
                    Some(i.object.as_str())
                } else {
                    None
                }
            })
            .collect();

        if compiled_objects.len() < orig_objects.len() {
            return Err(VerificationFailure {
                pass_name: self.name().to_string(),
                reason: "Instructions were removed during strengthening".to_string(),
            });
        }
        Ok(())
    }
}

impl InstructionStrengtheningPass {
    fn get_imperative(&self, verb: &InstructionVerb) -> &'static str {
        match verb {
            InstructionVerb::Write => "Write",
            InstructionVerb::Generate => "Generate",
            InstructionVerb::Analyze => "Analyze",
            InstructionVerb::Explain => "Explain",
            InstructionVerb::Summarize => "Summarize",
            InstructionVerb::Extract => "Extract",
            InstructionVerb::Classify => "Classify",
            InstructionVerb::Compare => "Compare",
            InstructionVerb::Translate => "Translate",
            InstructionVerb::Rewrite => "Rewrite",
            InstructionVerb::Design => "Design",
            InstructionVerb::Optimize => "Optimize",
            InstructionVerb::Debug => "Debug",
            InstructionVerb::Calculate => "Calculate",
            InstructionVerb::Convert => "Convert",
            InstructionVerb::Search => "Search",
            _ => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_strengthen_weak_instruction() {
        let mut ast = PromptRoot::new(vec![
            PromptNode::Instruction(Instruction {
                verb: InstructionVerb::Write,
                object: "Could you write a poem?".to_string(),
                modifiers: Vec::new(),
                confidence: 0.7,
                span: crate::ast::SourceSpan::new(Position::new(1, 1), Position::new(1, 25)),
            }),
        ]);
        let pass = InstructionStrengtheningPass;
        let ctx = PassContext {
            model_profile: None,
            annotations: Annotations::default(),
            config: UserConfig::default(),
            target_model: "claude-3.5-sonnet".to_string(),
        };
        let result = pass.run(&mut ast, &ctx).await.unwrap();
        assert!(result.applied);
    }

    #[tokio::test]
    async fn test_strong_instruction_unchanged() {
        let mut ast = PromptRoot::new(vec![
            PromptNode::Instruction(Instruction {
                verb: InstructionVerb::Write,
                object: "Write a poem about nature".to_string(),
                modifiers: Vec::new(),
                confidence: 0.9,
                span: crate::ast::SourceSpan::new(Position::new(1, 1), Position::new(1, 25)),
            }),
        ]);
        let pass = InstructionStrengtheningPass;
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
