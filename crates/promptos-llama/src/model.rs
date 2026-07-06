use crate::bridge::{InferenceConfig, InferenceOutput, ModelInfo};
use crate::inference::run_inference;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(120);

pub struct ModelManager {
    active: Mutex<Option<ActiveModel>>,
    config_path: String,
}

struct ActiveModel {
    model_path: String,
    last_used: Instant,
    sha256: String,
    size_bytes: u64,
}

impl ModelManager {
    pub fn new(model_path: &str) -> Self {
        Self {
            active: Mutex::new(None),
            config_path: model_path.to_string(),
        }
    }

    pub fn load(&self) -> Result<ModelInfo, String> {
        self.load_model(&self.config_path)
    }

    pub fn load_model(&self, path: &str) -> Result<ModelInfo, String> {
        let p = Path::new(path);
        if !p.exists() {
            return Err(format!("Model file not found: {}", path));
        }
        let metadata = std::fs::metadata(p).map_err(|e| format!("Cannot read model metadata: {}", e))?;
        let size_bytes = metadata.len();
        let sha = self.verify_integrity(path)?;

        let info = ModelInfo {
            path: path.to_string(),
            size_bytes,
            context_length: 4096,
            quantization: "Q4_K_M".to_string(),
        };

        let mut active = self.active.lock().map_err(|e| e.to_string())?;
        *active = Some(ActiveModel {
            model_path: path.to_string(),
            last_used: Instant::now(),
            sha256: sha,
            size_bytes,
        });

        tracing::info!("Model loaded: {} ({})", path, format_size(size_bytes));
        Ok(info)
    }

    pub fn unload(&self) -> Result<(), String> {
        let mut active = self.active.lock().map_err(|e| e.to_string())?;
        active.take();
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.active.lock().map(|a| a.is_some()).unwrap_or(false)
    }

    pub fn touch(&self) {
        if let Ok(mut active) = self.active.lock() {
            if let Some(ref mut am) = *active {
                am.last_used = Instant::now();
            }
        }
    }

    pub fn infer(&self, input: &str, config: &InferenceConfig) -> Result<InferenceOutput, String> {
        let path = {
            let active = self.active.lock().map_err(|e| e.to_string())?;
            match active.as_ref() {
                Some(am) => {
                    let path = am.model_path.clone();
                    drop(active);
                    self.update_timestamp();
                    path
                }
                None => return Err("Model not loaded".to_string()),
            }
        };

        run_inference(&path, input, config)
    }

    fn update_timestamp(&self) {
        if let Ok(mut active) = self.active.lock() {
            if let Some(ref mut am) = *active {
                am.last_used = Instant::now();
            }
        }
    }

    pub fn verify_integrity(&self, path: &str) -> Result<String, String> {
        let p = Path::new(path);
        if !p.exists() {
            return Err(format!("Model file not found: {}", path));
        }
        let mut file = std::fs::File::open(p).map_err(|e| format!("Cannot open model: {}", e))?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = file.read(&mut buf).map_err(|e| format!("Error reading model: {}", e))?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_manager_init() {
        let manager = ModelManager::new("/tmp/test.gguf");
        assert!(!manager.is_loaded());
    }

    #[test]
    fn test_load_nonexistent() {
        let manager = ModelManager::new("/nonexistent/model.gguf");
        let result = manager.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_nonexistent() {
        let manager = ModelManager::new("/nonexistent.gguf");
        assert!(manager.verify_integrity("/nonexistent.gguf").is_err());
    }
}
