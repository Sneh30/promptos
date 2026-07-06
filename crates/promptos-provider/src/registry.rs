use crate::traits::*;
use std::collections::HashMap;

pub struct ProviderRegistry {
    providers: HashMap<ProviderId, Box<dyn ModelProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn ModelProvider>) {
        let id = provider.id();
        self.providers.insert(id, provider);
    }

    pub fn register_defaults(&mut self) {
        self.register(Box::new(crate::AnthropicProvider::new()));
        self.register(Box::new(crate::OpenAIProvider::new()));
        self.register(Box::new(crate::GoogleProvider::new()));
    }

    pub fn get(&self, id: &ProviderId) -> Option<&Box<dyn ModelProvider>> {
        self.providers.get(id)
    }

    pub fn providers(&self) -> Vec<&Box<dyn ModelProvider>> {
        self.providers.values().collect()
    }

    pub fn provider_ids(&self) -> Vec<ProviderId> {
        self.providers.keys().copied().collect()
    }

    pub fn resolve_provider(&self, model_id: &str) -> Option<(ProviderId, &Box<dyn ModelProvider>)> {
        for (id, provider) in &self.providers {
            if provider.supported_models().iter().any(|m| m == model_id) {
                return Some((*id, provider));
            }
            if model_id.contains(id.as_str()) {
                return Some((*id, provider));
            }
        }
        None
    }

    pub async fn send_prompt(
        &self,
        model_id: &str,
        prompt: &CompiledPrompt,
        key: &ApiKey,
    ) -> Result<ModelResponse, ProviderError> {
        let (_id, provider) = self
            .resolve_provider(model_id)
            .ok_or_else(|| ProviderError::ModelUnavailable(format!("No provider for model: {}", model_id)))?;

        provider.send_prompt(prompt, key).await
    }

    pub fn estimate_cost(&self, model_id: &str, prompt: &CompiledPrompt) -> Option<CostEstimate> {
        self.resolve_provider(model_id)
            .map(|(_, provider)| provider.estimate_cost(prompt))
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
