use crate::traits::*;
use async_trait::async_trait;

pub struct AnthropicProvider;

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self
    }

    fn api_url(&self) -> &str {
        "https://api.anthropic.com/v1/messages"
    }

    fn model_name(&self, model_id: &str) -> String {
        match model_id {
            "claude-3.5-sonnet" => "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-opus" => "claude-3-opus-20240229".to_string(),
            "claude-3-haiku" => "claude-3-haiku-20240307".to_string(),
            _ => model_id.to_string(),
        }
    }
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Anthropic
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "claude-3.5-sonnet".to_string(),
            "claude-3-opus".to_string(),
            "claude-3-haiku".to_string(),
        ]
    }

    async fn send_prompt(
        &self,
        prompt: &CompiledPrompt,
        key: &ApiKey,
    ) -> Result<ModelResponse, ProviderError> {
        let model = self.model_name(&prompt.model_id);
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "model": model,
            "max_tokens": prompt.max_output_tokens.unwrap_or(8192),
            "messages": [
                {"role": "user", "content": prompt.text}
            ]
        });

        let start = std::time::Instant::now();
        let response = client
            .post(self.api_url())
            .header("x-api-key", key.as_str())
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return match status.as_u16() {
                401 => Err(ProviderError::Authentication(text)),
                429 => Err(ProviderError::RateLimited { retry_after: None }),
                _ => Err(ProviderError::InvalidRequest(format!(
                    "Status {}: {}",
                    status, text
                ))),
            };
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ProviderError::Internal(format!("Parse error: {}", e)))?;

        let text = raw["content"][0]["text"].as_str().unwrap_or("").to_string();
        let input_tokens = raw["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = raw["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(ModelResponse {
            text,
            model_id: prompt.model_id.clone(),
            input_tokens,
            output_tokens,
            latency_ms,
            finish_reason: "stop".to_string(),
            raw_response: raw,
        })
    }

    fn estimate_cost(&self, prompt: &CompiledPrompt) -> CostEstimate {
        let input_cost = (prompt.text.split_whitespace().count() as f64 * 3.00) / 1_000_000.0;
        let output_cost = (prompt.max_output_tokens.unwrap_or(8192) as f64 * 15.00) / 1_000_000.0;
        CostEstimate {
            input_cost,
            output_cost,
            total_cost: input_cost + output_cost,
            currency: "USD".to_string(),
        }
    }

    async fn validate_key(&self, key: &ApiKey) -> Result<bool, ProviderError> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.anthropic.com/v1/models")
            .header("x-api-key", key.as_str())
            .header("anthropic-version", "2023-06-01")
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        Ok(response.status().is_success())
    }
}
