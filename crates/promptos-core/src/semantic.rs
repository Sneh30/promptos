use crate::ast::*;
use async_trait::async_trait;

#[derive(Debug)]
pub enum AnalysisError {
    ModelUnavailable(String),
    ParseError(String),
    Timeout,
    Internal(String),
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelUnavailable(msg) => write!(f, "Model unavailable: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::Timeout => write!(f, "Analysis timed out"),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AnalysisError {}

#[async_trait]
pub trait SemanticAnalyzer: Send + Sync {
    async fn analyze(&self, ast: &mut PromptRoot) -> Result<Annotations, AnalysisError>;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct PassContext {
    pub model_profile: Option<ModelProfileData>,
    pub annotations: Annotations,
    pub config: UserConfig,
    pub target_model: String,
}

#[derive(Debug, Clone)]
pub struct ModelProfileData {
    pub model_id: String,
    pub provider: String,
    pub version: String,
    pub context_limit_input: u32,
    pub context_limit_output: u32,
    pub max_output_tokens: u32,
    pub pricing_input_per_mtok: f64,
    pub pricing_output_per_mtok: f64,
}

pub struct RuleBasedAnalyzer;

impl RuleBasedAnalyzer {
    fn extract_intent(&self, ast: &PromptRoot) -> Option<Intent> {
        for child in &ast.children {
            if let PromptNode::Instruction(instr) = child {
                let complexity = match instr.verb {
                    InstructionVerb::Analyze
                    | InstructionVerb::Explain
                    | InstructionVerb::Compare => Complexity::Moderate,
                    InstructionVerb::Design | InstructionVerb::Optimize => Complexity::Complex,
                    _ => Complexity::Simple,
                };
                return Some(Intent {
                    primary_task: instr.object.clone(),
                    domain: None,
                    output_type: OutputType::Text,
                    complexity,
                });
            }
        }
        None
    }

    fn detect_ambiguities(&self, ast: &PromptRoot) -> Vec<Ambiguity> {
        let mut ambiguities = Vec::new();
        for child in &ast.children {
            match child {
                PromptNode::Instruction(instr) => {
                    if instr.object.contains("it") || instr.object.contains("this") {
                        ambiguities.push(Ambiguity {
                            text: instr.object.clone(),
                            span: instr.span,
                            interpretations: vec![
                                "Referent is ambiguous, could refer to multiple subjects"
                                    .to_string(),
                            ],
                            recommended_resolution: Some(
                                "Replace pronoun with explicit noun".to_string(),
                            ),
                            confidence: 0.6,
                        });
                    }
                }
                PromptNode::Context(ctx) => {
                    if ctx.content.len() < 10 && !ctx.content.is_empty() {
                        ambiguities.push(Ambiguity {
                            text: ctx.content.clone(),
                            span: ctx.span,
                            interpretations: vec![
                                "Context too short to be meaningful".to_string(),
                                "May be a fragment".to_string(),
                            ],
                            recommended_resolution: Some("Provide more context".to_string()),
                            confidence: 0.4,
                        });
                    }
                }
                _ => {}
            }
        }
        ambiguities
    }

    fn detect_contradictions(&self, ast: &PromptRoot) -> Vec<Contradiction> {
        let mut contradictions = Vec::new();
        let constraints: Vec<&Constraint> = ast
            .children
            .iter()
            .filter_map(|c| {
                if let PromptNode::Constraint(cn) = c {
                    Some(cn)
                } else {
                    None
                }
            })
            .collect();

        for (i, ca) in constraints.iter().enumerate() {
            for cb in constraints.iter().skip(i + 1) {
                if ca.constraint_type != cb.constraint_type
                    && ca.value.to_lowercase().contains("concise")
                    && cb.value.to_lowercase().contains("detailed")
                {
                    contradictions.push(Contradiction {
                        constraint_a: ca.span,
                        constraint_b: cb.span,
                        description: "Concise and detailed are contradictory constraints"
                            .to_string(),
                    });
                }
                if ca.value.to_lowercase().contains("json")
                    && cb.value.to_lowercase().contains("markdown")
                {
                    contradictions.push(Contradiction {
                        constraint_a: ca.span,
                        constraint_b: cb.span,
                        description: "JSON and Markdown output formats are contradictory"
                            .to_string(),
                    });
                }
            }
        }
        contradictions
    }

    fn detect_context_gaps(&self, ast: &PromptRoot) -> Vec<ContextGap> {
        let mut gaps = Vec::new();
        let has_instruction = ast
            .children
            .iter()
            .any(|c| matches!(c, PromptNode::Instruction(_)));
        let has_context = ast
            .children
            .iter()
            .any(|c| matches!(c, PromptNode::Context(_)));

        if !has_instruction {
            gaps.push(ContextGap {
                gap_type: GapType::MissingInstruction,
                description: "No explicit instruction detected".to_string(),
                suggested_addition: Some(
                    "Add a clear instruction specifying what the model should do".to_string(),
                ),
            });
        }

        if !has_context {
            gaps.push(ContextGap {
                gap_type: GapType::MissingContext,
                description: "No background context provided".to_string(),
                suggested_addition: Some("Add relevant context for the task".to_string()),
            });
        }

        gaps
    }
}

#[async_trait]
impl SemanticAnalyzer for RuleBasedAnalyzer {
    fn name(&self) -> &str {
        "rule-based-analyzer"
    }

    async fn analyze(&self, ast: &mut PromptRoot) -> Result<Annotations, AnalysisError> {
        let intent = self.extract_intent(ast);
        let ambiguities = self.detect_ambiguities(ast);
        let contradictions = self.detect_contradictions(ast);
        let context_gaps = self.detect_context_gaps(ast);

        let original_text = ast
            .children
            .iter()
            .map(|c| format!("{:?}", c))
            .collect::<Vec<_>>()
            .join(" ");
        let token_count = original_text.split_whitespace().count();

        let annotations = Annotations {
            intent,
            detected_ambiguities: ambiguities,
            detected_contradictions: contradictions,
            detected_context_gaps: context_gaps,
            dependencies: Vec::new(),
            optimization_log: Vec::new(),
            token_count_original: token_count,
            token_count_compiled: token_count,
            estimated_cost_original: 0.0,
            estimated_cost_compiled: 0.0,
            estimated_latency_original: 0.0,
            estimated_latency_compiled: 0.0,
            quality_score_original: 5.0,
            quality_score_compiled: 5.0,
            hallucination_risk: 0.0,
            diagnostics: Vec::new(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        ast.annotations = annotations.clone();
        Ok(annotations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[tokio::test]
    async fn test_rule_based_analyzer() {
        let input = "Write a Python function that sorts an array. The output should be concise.";
        let mut root = parser::parse(input).unwrap();
        let analyzer = RuleBasedAnalyzer;
        let result = analyzer.analyze(&mut root).await;
        assert!(result.is_ok());
        let annotations = result.unwrap();
        assert!(annotations.token_count_original > 0);
    }

    #[tokio::test]
    async fn test_ambiguity_detection() {
        let input = "Analyze it and return results.";
        let mut root = parser::parse(input).unwrap();
        let analyzer = RuleBasedAnalyzer;
        let result = analyzer.analyze(&mut root).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_gap_detection() {
        let input = "Explain quantum computing.";
        let mut root = parser::parse(input).unwrap();
        let analyzer = RuleBasedAnalyzer;
        let annotations = analyzer.analyze(&mut root).await.unwrap();
        let gaps = &annotations.detected_context_gaps;
        assert!(!gaps.is_empty());
    }
}
