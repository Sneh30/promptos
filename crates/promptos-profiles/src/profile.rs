use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub profile: ProfileMeta,
    pub specs: Specs,
    pub performance: Performance,
    pub format_reliability: FormatReliability,
    pub behavior: Behavior,
    pub safety: Safety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    pub model_id: String,
    pub provider: String,
    pub version: String,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specs {
    pub context_limit_input: u32,
    pub context_limit_output: u32,
    pub max_output_tokens: u32,
    pub pricing_input_per_mtok: f64,
    pub pricing_output_per_mtok: f64,
    pub pricing_cached_input_per_mtok: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Performance {
    pub reasoning_quality: f32,
    pub coding_quality: f32,
    pub writing_quality: f32,
    pub analysis_quality: f32,
    pub structured_extraction_quality: f32,
    pub tool_use_reliability: f32,
    pub instruction_following: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatReliability {
    pub xml: f32,
    pub markdown: f32,
    pub json_structured: f32,
    pub json_unstructured: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Behavior {
    pub chain_of_thought_effective: bool,
    pub self_correction_tendency: f32,
    pub verbose_conciseness: f32,
    pub long_context_retrieval_accuracy: f32,
    pub primacy_effect_bias: f32,
    pub recency_effect_bias: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Safety {
    pub hallucination_risk_general: f32,
    pub hallucination_risk_coding: f32,
    pub hallucination_risk_factual: f32,
    pub refusal_rate: f32,
    pub sycophancy_tendency: f32,
}

pub fn anthropic_claude_3_5_sonnet() -> ModelProfile {
    ModelProfile {
        profile: ProfileMeta {
            model_id: "claude-3.5-sonnet".to_string(),
            provider: "anthropic".to_string(),
            version: "1.2.0".to_string(),
            last_updated: "2026-01-15".to_string(),
        },
        specs: Specs {
            context_limit_input: 200000,
            context_limit_output: 8192,
            max_output_tokens: 8192,
            pricing_input_per_mtok: 3.00,
            pricing_output_per_mtok: 15.00,
            pricing_cached_input_per_mtok: 0.30,
        },
        performance: Performance {
            reasoning_quality: 0.92,
            coding_quality: 0.89,
            writing_quality: 0.90,
            analysis_quality: 0.91,
            structured_extraction_quality: 0.88,
            tool_use_reliability: 0.85,
            instruction_following: 0.93,
        },
        format_reliability: FormatReliability {
            xml: 0.95,
            markdown: 0.90,
            json_structured: 0.85,
            json_unstructured: 0.75,
        },
        behavior: Behavior {
            chain_of_thought_effective: true,
            self_correction_tendency: 0.3,
            verbose_conciseness: 0.5,
            long_context_retrieval_accuracy: 0.85,
            primacy_effect_bias: 0.2,
            recency_effect_bias: 0.3,
        },
        safety: Safety {
            hallucination_risk_general: 0.08,
            hallucination_risk_coding: 0.05,
            hallucination_risk_factual: 0.12,
            refusal_rate: 0.02,
            sycophancy_tendency: 0.3,
        },
    }
}

pub fn openai_gpt4o() -> ModelProfile {
    ModelProfile {
        profile: ProfileMeta {
            model_id: "gpt-4o".to_string(),
            provider: "openai".to_string(),
            version: "1.1.0".to_string(),
            last_updated: "2026-01-10".to_string(),
        },
        specs: Specs {
            context_limit_input: 128000,
            context_limit_output: 16384,
            max_output_tokens: 16384,
            pricing_input_per_mtok: 5.00,
            pricing_output_per_mtok: 15.00,
            pricing_cached_input_per_mtok: 2.50,
        },
        performance: Performance {
            reasoning_quality: 0.91,
            coding_quality: 0.93,
            writing_quality: 0.88,
            analysis_quality: 0.90,
            structured_extraction_quality: 0.90,
            tool_use_reliability: 0.88,
            instruction_following: 0.91,
        },
        format_reliability: FormatReliability {
            xml: 0.70,
            markdown: 0.95,
            json_structured: 0.92,
            json_unstructured: 0.85,
        },
        behavior: Behavior {
            chain_of_thought_effective: true,
            self_correction_tendency: 0.4,
            verbose_conciseness: 0.4,
            long_context_retrieval_accuracy: 0.80,
            primacy_effect_bias: 0.3,
            recency_effect_bias: 0.4,
        },
        safety: Safety {
            hallucination_risk_general: 0.07,
            hallucination_risk_coding: 0.04,
            hallucination_risk_factual: 0.10,
            refusal_rate: 0.03,
            sycophancy_tendency: 0.25,
        },
    }
}

pub fn google_gemini_1_5_pro() -> ModelProfile {
    ModelProfile {
        profile: ProfileMeta {
            model_id: "gemini-1.5-pro".to_string(),
            provider: "google".to_string(),
            version: "1.0.0".to_string(),
            last_updated: "2026-01-08".to_string(),
        },
        specs: Specs {
            context_limit_input: 1000000,
            context_limit_output: 8192,
            max_output_tokens: 8192,
            pricing_input_per_mtok: 3.50,
            pricing_output_per_mtok: 10.50,
            pricing_cached_input_per_mtok: 0.35,
        },
        performance: Performance {
            reasoning_quality: 0.88,
            coding_quality: 0.85,
            writing_quality: 0.87,
            analysis_quality: 0.89,
            structured_extraction_quality: 0.86,
            tool_use_reliability: 0.80,
            instruction_following: 0.87,
        },
        format_reliability: FormatReliability {
            xml: 0.75,
            markdown: 0.88,
            json_structured: 0.90,
            json_unstructured: 0.82,
        },
        behavior: Behavior {
            chain_of_thought_effective: true,
            self_correction_tendency: 0.35,
            verbose_conciseness: 0.45,
            long_context_retrieval_accuracy: 0.90,
            primacy_effect_bias: 0.15,
            recency_effect_bias: 0.25,
        },
        safety: Safety {
            hallucination_risk_general: 0.10,
            hallucination_risk_coding: 0.08,
            hallucination_risk_factual: 0.15,
            refusal_rate: 0.01,
            sycophancy_tendency: 0.35,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_profile() {
        let profile = anthropic_claude_3_5_sonnet();
        assert_eq!(profile.profile.model_id, "claude-3.5-sonnet");
        assert_eq!(profile.profile.provider, "anthropic");
        assert_eq!(profile.specs.context_limit_input, 200000);
    }

    #[test]
    fn test_openai_profile() {
        let profile = openai_gpt4o();
        assert_eq!(profile.profile.model_id, "gpt-4o");
        assert_eq!(profile.specs.context_limit_input, 128000);
    }

    #[test]
    fn test_google_profile() {
        let profile = google_gemini_1_5_pro();
        assert_eq!(profile.profile.model_id, "gemini-1.5-pro");
        assert_eq!(profile.specs.context_limit_input, 1000000);
    }
}
