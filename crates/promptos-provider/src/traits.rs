use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(key: &str) -> Self {
        Self(key.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() > 8 {
            write!(f, "{}...{}", &self.0[..4], &self.0[self.0.len() - 4..])
        } else {
            write!(f, "****")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderId {
    Anthropic,
    OpenAI,
    Google,
}

impl ProviderId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenAI => "openai",
            Self::Google => "google",
        }
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPrompt {
    pub text: String,
    pub model_id: String,
    pub mode: String,
    pub max_output_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub structured_output_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub text: String,
    pub model_id: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub latency_ms: u64,
    pub finish_reason: String,
    pub raw_response: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub input_cost: f64,
    pub output_cost: f64,
    pub total_cost: f64,
    pub currency: String,
}

#[derive(Debug)]
pub enum ProviderError {
    Authentication(String),
    RateLimited { retry_after: Option<u64> },
    QuotaExceeded(String),
    ModelUnavailable(String),
    InvalidRequest(String),
    Timeout,
    Network(String),
    Internal(String),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            Self::RateLimited { retry_after } => {
                write!(f, "Rate limited (retry after: {:?})", retry_after)
            }
            Self::QuotaExceeded(msg) => write!(f, "Quota exceeded: {}", msg),
            Self::ModelUnavailable(msg) => write!(f, "Model unavailable: {}", msg),
            Self::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            Self::Timeout => write!(f, "Request timed out"),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ProviderError {}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn name(&self) -> &str;
    fn supported_models(&self) -> Vec<String>;
    async fn send_prompt(
        &self,
        prompt: &CompiledPrompt,
        key: &ApiKey,
    ) -> Result<ModelResponse, ProviderError>;
    fn estimate_cost(&self, prompt: &CompiledPrompt) -> CostEstimate;
    async fn validate_key(&self, key: &ApiKey) -> Result<bool, ProviderError>;
}
