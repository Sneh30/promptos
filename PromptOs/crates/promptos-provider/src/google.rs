use crate::traits::*;
use async_trait::async_trait;

pub struct GoogleProvider;

impl Default for GoogleProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleProvider {
    pub fn new() -> Self {
        Self
    }

    fn api_url(&self, model_id: &str) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1/models/{}:generateContent",
            self.model_name(model_id)
        )
    }

    fn model_name(&self, model_id: &str) -> String {
        match model_id {
            "gemini-1.5-pro" => "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash" => "gemini-1.5-flash".to_string(),
            _ => model_id.to_string(),
        }
    }
}

#[async_trait]
impl ModelProvider for GoogleProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Google
    }

    fn name(&self) -> &str {
        "Google"
    }

    fn supported_models(&self) -> Vec<String> {
        vec!["gemini-1.5-pro".to_string(), "gemini-1.5-flash".to_string()]
    }

    async fn send_prompt(
        &self,
        prompt: &CompiledPrompt,
        key: &ApiKey,
    ) -> Result<ModelResponse, ProviderError> {
        let client = reqwest::Client::new();
        let url = format!("{}?key={}", self.api_url(&prompt.model_id), key.as_str());

        let body = serde_json::json!({
            "contents": [
                {
                    "parts": [
                        {"text": prompt.text}
                    ]
                }
            ],
            "generationConfig": {
                "maxOutputTokens": prompt.max_output_tokens.unwrap_or(8192),
            }
        });

        let start = std::time::Instant::now();
        let response = client
            .post(&url)
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
                401 | 403 => Err(ProviderError::Authentication(text)),
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

        let text = raw["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(ModelResponse {
            text,
            model_id: prompt.model_id.clone(),
            input_tokens: 0,
            output_tokens: 0,
            latency_ms,
            finish_reason: "stop".to_string(),
            raw_response: raw,
        })
    }

    fn estimate_cost(&self, prompt: &CompiledPrompt) -> CostEstimate {
        let input_cost = (prompt.text.split_whitespace().count() as f64 * 3.50) / 1_000_000.0;
        let output_cost = (prompt.max_output_tokens.unwrap_or(8192) as f64 * 10.50) / 1_000_000.0;
        CostEstimate {
            input_cost,
            output_cost,
            total_cost: input_cost + output_cost,
            currency: "USD".to_string(),
        }
    }

    async fn validate_key(&self, key: &ApiKey) -> Result<bool, ProviderError> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1/models?key={}",
            key.as_str()
        );
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        Ok(response.status().is_success())
    }
}
