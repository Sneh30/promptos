use crate::traits::*;
use async_trait::async_trait;

pub struct OpenAIProvider;

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAIProvider {
    pub fn new() -> Self {
        Self
    }

    fn api_url(&self) -> &str {
        "https://api.openai.com/v1/chat/completions"
    }

    fn model_name(&self, model_id: &str) -> String {
        match model_id {
            "gpt-4o" => "gpt-4o".to_string(),
            "gpt-4o-mini" => "gpt-4o-mini".to_string(),
            "o1" => "o1".to_string(),
            "o3" => "o3".to_string(),
            _ => model_id.to_string(),
        }
    }
}

#[async_trait]
impl ModelProvider for OpenAIProvider {
    fn id(&self) -> ProviderId {
        ProviderId::OpenAI
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "o1".to_string(),
            "o3".to_string(),
        ]
    }

    async fn send_prompt(
        &self,
        prompt: &CompiledPrompt,
        key: &ApiKey,
    ) -> Result<ModelResponse, ProviderError> {
        let model = self.model_name(&prompt.model_id);
        let client = reqwest::Client::new();

        let mut body = serde_json::json!({
            "model": model,
            "max_tokens": prompt.max_output_tokens.unwrap_or(16384),
            "messages": [
                {"role": "user", "content": prompt.text}
            ]
        });

        if let Some(temp) = prompt.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        let start = std::time::Instant::now();
        let response = client
            .post(self.api_url())
            .header("Authorization", format!("Bearer {}", key.as_str()))
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

        let text = raw["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let input_tokens = raw["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = raw["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(ModelResponse {
            text,
            model_id: prompt.model_id.clone(),
            input_tokens,
            output_tokens,
            latency_ms,
            finish_reason: raw["choices"][0]["finish_reason"]
                .as_str()
                .unwrap_or("stop")
                .to_string(),
            raw_response: raw,
        })
    }

    fn estimate_cost(&self, prompt: &CompiledPrompt) -> CostEstimate {
        let input_cost = (prompt.text.split_whitespace().count() as f64 * 5.00) / 1_000_000.0;
        let output_cost = (prompt.max_output_tokens.unwrap_or(16384) as f64 * 15.00) / 1_000_000.0;
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
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", key.as_str()))
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        Ok(response.status().is_success())
    }
}
