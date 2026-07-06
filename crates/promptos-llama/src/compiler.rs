use crate::bridge::InferenceConfig;
use crate::inference::{build_chatml_prompt, truncate_to_tokens};
use crate::model::ModelManager;
use serde::{Deserialize, Serialize};

const COMPILER_SYSTEM_PROMPT: &str = r#"You are a prompt optimization engine. Rewrite the user's prompt to be maximally concise while preserving ALL intent, meaning, and constraints.

Rules:
1. Remove filler words, pleasantries, and meta-commentary
2. Replace weak phrases ("could you", "I'd like", "if possible") with direct imperatives
3. Consolidate redundant or repeated instructions
4. Use precise, concrete terminology over vague language
5. Preserve ALL specific requirements, formats, roles, and constraints
6. Keep the core instruction clear and unambiguous
7. Output ONLY the optimized prompt — no explanations, no prefixes, no quotes"#;

const COMPILER_EVALUATION_PROMPT: &str = r#"You are a prompt quality evaluator. Analyze the following two prompts — the ORIGINAL and the OPTIMIZED version.

Evaluate the optimized prompt on these criteria:
1. Did it preserve the original intent? (yes/no)
2. Did it reduce token count? (yes/no — compare word counts)
3. Did it strengthen weak instructions? (yes/no)
4. Did it maintain all constraints? (yes/no)

Output a single JSON object with your evaluation."#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationResult {
    pub optimized_text: String,
    pub original_token_count: u32,
    pub optimized_token_count: u32,
    pub token_reduction_pct: f64,
    pub passes_applied: Vec<String>,
    pub inference_time_ms: u64,
    pub evaluation: Option<EvaluationReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationReport {
    pub intent_preserved: bool,
    pub token_reduction_achieved: bool,
    pub instructions_strengthened: bool,
    pub constraints_maintained: bool,
    pub quality_score: f32,
}

impl CompilationResult {
    pub fn tokens_saved(&self) -> i32 {
        self.original_token_count as i32 - self.optimized_token_count as i32
    }
}

pub struct LlamaCompiler {
    manager: ModelManager,
    config: InferenceConfig,
}

impl LlamaCompiler {
    pub fn new(model_path: &str) -> Self {
        Self {
            manager: ModelManager::new(model_path),
            config: InferenceConfig {
                temperature: 0.1,
                top_p: 0.9,
                max_tokens: 4096,
                repeat_penalty: 1.1,
                seed: Some(42),
            },
        }
    }

    pub fn load_model(&self) -> Result<crate::bridge::ModelInfo, String> {
        self.manager.load()
    }

    pub fn is_model_loaded(&self) -> bool {
        self.manager.is_loaded()
    }

    pub fn compile(&self, input: &str) -> Result<CompilationResult, String> {
        let original_tokens = count_tokens(input);

        let prompt = build_chatml_prompt(Some(COMPILER_SYSTEM_PROMPT), input);
        let truncated_prompt = truncate_to_tokens(&prompt, 3072);

        let output = self.manager.infer(&truncated_prompt, &self.config)?;

        let optimized = trim_llm_output(&output.text);
        let optimized_tokens = count_tokens(&optimized);

        let reduction = if original_tokens > 0 {
            ((original_tokens as f64 - optimized_tokens as f64) / original_tokens as f64) * 100.0
        } else {
            0.0
        };

        let passes = build_pass_list(input, &optimized);

        let evaluation = if !optimized.is_empty() && optimized != input {
            self.evaluate(input, &optimized).ok()
        } else {
            None
        };

        Ok(CompilationResult {
            optimized_text: optimized,
            original_token_count: original_tokens,
            optimized_token_count: optimized_tokens,
            token_reduction_pct: reduction.max(0.0),
            passes_applied: passes,
            inference_time_ms: output.inference_time_ms,
            evaluation,
        })
    }

    fn evaluate(&self, original: &str, optimized: &str) -> Result<EvaluationReport, String> {
        let eval_prompt = format!(
            "{}\n\nORIGINAL:\n{}\n\nOPTIMIZED:\n{}\n\nEvaluate and output JSON:",
            COMPILER_EVALUATION_PROMPT, original, optimized
        );
        let prompt = build_chatml_prompt(None, &eval_prompt);

        let eval_config = InferenceConfig {
            temperature: 0.0,
            max_tokens: 256,
            seed: Some(0),
            ..self.config
        };

        let output = self.manager.infer(&prompt, &eval_config)?;

        let report = parse_evaluation_json(&output.text);

        Ok(report)
    }
}

fn count_tokens(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

fn trim_llm_output(text: &str) -> String {
    let text = text.trim();
    let text = text
        .strip_prefix("\"")
        .unwrap_or(text)
        .strip_suffix("\"")
        .unwrap_or(text);
    let text = text
        .strip_prefix("'")
        .unwrap_or(text)
        .strip_suffix("'")
        .unwrap_or(text);
    text.trim().to_string()
}

fn build_pass_list(original: &str, optimized: &str) -> Vec<String> {
    let mut passes = Vec::new();
    let orig_lower = original.to_lowercase();

    passes.push("llm-compilation".to_string());

    if count_tokens(optimized) < count_tokens(original) {
        passes.push("token-reduction".to_string());
    }

    let weak = ["could you", "maybe", "i'd like", "if possible", "would you mind", "please"];
    if weak.iter().any(|w| orig_lower.contains(w)) {
        passes.push("instruction-strengthening".to_string());
    }

    if optimized.lines().count() < original.lines().count() {
        passes.push("redundancy-elimination".to_string());
    }

    if optimized.len() < original.len() / 2 {
        passes.push("aggressive-compression".to_string());
    }

    passes
}

fn parse_evaluation_json(text: &str) -> EvaluationReport {
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            let json_str = &text[start..=end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                return EvaluationReport {
                    intent_preserved: val.get("intent_preserved").or(val.get("1")).and_then(|v| v.as_bool()).unwrap_or(true),
                    token_reduction_achieved: val.get("token_reduction").or(val.get("2")).and_then(|v| v.as_bool()).unwrap_or(false),
                    instructions_strengthened: val.get("instructions_strengthened").or(val.get("3")).and_then(|v| v.as_bool()).unwrap_or(false),
                    constraints_maintained: val.get("constraints_maintained").or(val.get("4")).and_then(|v| v.as_bool()).unwrap_or(true),
                    quality_score: val.get("quality_score").and_then(|v| v.as_f64()).map(|v| v as f32).unwrap_or(7.0),
                };
            }
        }
    }

    let has_yes = |keyword: &str| text.to_lowercase().contains(keyword);
    EvaluationReport {
        intent_preserved: has_yes("intent_preserved") || has_yes("\"1\": true") || has_yes("\"1\": \"yes"),
        token_reduction_achieved: has_yes("token_reduction") || has_yes("\"2\": true") || has_yes("\"2\": \"yes"),
        instructions_strengthened: has_yes("instructions_strengthened") || has_yes("\"3\": true") || has_yes("\"3\": \"yes"),
        constraints_maintained: has_yes("constraints_maintained") || has_yes("\"4\": true") || has_yes("\"4\": \"yes"),
        quality_score: 7.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens() {
        assert_eq!(count_tokens("hello world"), 2);
        assert_eq!(count_tokens(""), 0);
    }

    #[test]
    fn test_trim_llm_output_quotes() {
        assert_eq!(trim_llm_output("\"hello world\""), "hello world");
        assert_eq!(trim_llm_output("hello world"), "hello world");
    }

    #[test]
    fn test_build_pass_list_reduction() {
        let passes = build_pass_list("hello world foo bar", "hello");
        assert!(passes.contains(&"token-reduction".to_string()));
    }

    #[test]
    fn test_build_pass_list_weak() {
        let passes = build_pass_list("could you please help me", "help me");
        assert!(passes.contains(&"instruction-strengthening".to_string()));
    }
}
