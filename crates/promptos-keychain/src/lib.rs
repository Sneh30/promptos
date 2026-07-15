use security_framework::passwords::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeychainError {
    #[error("Keychain access error: {0}")]
    AccessError(String),
    #[error("Item not found")]
    NotFound,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

const SERVICE_NAME: &str = "com.promptos.app.api-keys";

pub struct KeychainManager;

impl KeychainManager {
    pub fn new() -> Self {
        Self
    }

    pub fn store_api_key(provider: &str, key: &str) -> Result<(), KeychainError> {
        if key.is_empty() {
            return Err(KeychainError::InvalidInput(
                "API key cannot be empty".to_string(),
            ));
        }
        if provider.is_empty() {
            return Err(KeychainError::InvalidInput(
                "Provider cannot be empty".to_string(),
            ));
        }

        set_generic_password(SERVICE_NAME, provider, key.as_bytes())
            .map_err(|e| KeychainError::AccessError(e.to_string()))
    }

    pub fn retrieve_api_key(provider: &str) -> Result<String, KeychainError> {
        let password = get_generic_password(SERVICE_NAME, provider).map_err(|e| {
            if e.code() == -25300 {
                KeychainError::NotFound
            } else {
                KeychainError::AccessError(e.to_string())
            }
        })?;

        String::from_utf8(password)
            .map_err(|_| KeychainError::AccessError("Invalid UTF-8 in stored key".to_string()))
    }

    pub fn delete_api_key(provider: &str) -> Result<(), KeychainError> {
        delete_generic_password(SERVICE_NAME, provider).map_err(|e| {
            if e.code() == -25300 {
                KeychainError::NotFound
            } else {
                KeychainError::AccessError(e.to_string())
            }
        })
    }

    pub fn has_key(provider: &str) -> Result<bool, KeychainError> {
        match Self::retrieve_api_key(provider) {
            Ok(_) => Ok(true),
            Err(KeychainError::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_empty_key() {
        let result = KeychainManager::store_api_key("test-provider", "");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            KeychainError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_store_empty_provider() {
        let result = KeychainManager::store_api_key("", "test-key");
        assert!(result.is_err());
    }

    #[test]
    fn test_has_nonexistent_key() {
        let result = KeychainManager::has_key("nonexistent-provider-12345");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
