use crate::traits::*;
use log::{debug, info, warn};
use std::collections::HashMap;

pub struct ProviderRegistry {
    providers: HashMap<ProviderId, Box<dyn ModelProvider>>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn ModelProvider>) {
        let id = provider.id();
        info!("Provider register — id={:?}, name={}", id, provider.name());
        self.providers.insert(id, provider);
    }

    pub fn register_defaults(&mut self) {
        info!("Provider register_defaults — registering Anthropic, OpenAI, Google");
        self.register(Box::new(crate::AnthropicProvider::new()));
        self.register(Box::new(crate::OpenAIProvider::new()));
        self.register(Box::new(crate::GoogleProvider::new()));
    }

    pub fn get(&self, id: &ProviderId) -> Option<&dyn ModelProvider> {
        self.providers.get(id).map(|b| &**b)
    }

    pub fn providers(&self) -> Vec<&dyn ModelProvider> {
        self.providers.values().map(|b| &**b).collect()
    }

    pub fn provider_ids(&self) -> Vec<ProviderId> {
        self.providers.keys().copied().collect()
    }

    pub fn resolve_provider(&self, model_id: &str) -> Option<(ProviderId, &dyn ModelProvider)> {
        for (id, provider) in &self.providers {
            if provider.supported_models().iter().any(|m| m == model_id) {
                debug!("Provider resolve — model={}, provider={:?}", model_id, id);
                return Some((*id, &**provider));
            }
            if model_id.contains(id.as_str()) {
                debug!(
                    "Provider resolve — model={} (contains), provider={:?}",
                    model_id, id
                );
                return Some((*id, &**provider));
            }
        }
        warn!(
            "Provider resolve — no provider found for model: {}",
            model_id
        );
        None
    }

    pub async fn send_prompt(
        &self,
        model_id: &str,
        prompt: &CompiledPrompt,
        key: &ApiKey,
    ) -> Result<ModelResponse, ProviderError> {
        let (_id, provider) = self.resolve_provider(model_id).ok_or_else(|| {
            ProviderError::ModelUnavailable(format!("No provider for model: {}", model_id))
        })?;

        info!(
            "Provider send_prompt — model={}, prompt_len={}, provider={:?}",
            model_id,
            prompt.text.len(),
            _id
        );
        let result = provider.send_prompt(prompt, key).await;
        match &result {
            Ok(resp) => info!(
                "Provider response — model={}, output_len={}, latency_ms={}, finish_reason={}",
                model_id, resp.output_tokens, resp.latency_ms, resp.finish_reason
            ),
            Err(e) => warn!("Provider error — model={}, error={:?}", model_id, e),
        }
        result
    }

    pub fn estimate_cost(&self, model_id: &str, prompt: &CompiledPrompt) -> Option<CostEstimate> {
        let cost = self
            .resolve_provider(model_id)
            .map(|(_, provider)| provider.estimate_cost(prompt));
        if let Some(ref c) = cost {
            debug!(
                "Provider estimate_cost — model={}, total_cost={} {}",
                model_id, c.total_cost, c.currency
            );
        }
        cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_empty() {
        let registry = ProviderRegistry::new();
        assert!(registry.providers().is_empty());
        assert!(registry.resolve_provider("claude-3.5-sonnet").is_none());
    }

    #[test]
    fn test_registry_with_defaults() {
        let mut registry = ProviderRegistry::new();
        registry.register_defaults();
        assert_eq!(registry.providers().len(), 3);
        assert!(registry.resolve_provider("claude-3.5-sonnet").is_some());
        assert!(registry.resolve_provider("gpt-4o").is_some());
        assert!(registry.resolve_provider("gemini-1.5-pro").is_some());
    }

    #[test]
    fn test_registry_resolve_unknown() {
        let mut registry = ProviderRegistry::new();
        registry.register_defaults();
        assert!(registry.resolve_provider("unknown-model").is_none());
    }
}
