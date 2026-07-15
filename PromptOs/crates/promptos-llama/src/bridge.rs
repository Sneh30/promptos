use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: u32,
    pub repeat_penalty: f32,
    pub seed: Option<u64>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            temperature: 0.1,
            top_p: 0.9,
            max_tokens: 2048,
            repeat_penalty: 1.1,
            seed: Some(42),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceOutput {
    pub text: String,
    pub tokens_generated: u32,
    pub tokens_input: u32,
    pub inference_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub path: String,
    pub size_bytes: u64,
    pub context_length: u32,
    pub quantization: String,
}
