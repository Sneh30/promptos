use log::{info, warn};
use std::path::{Path, PathBuf};

const DEFAULT_MODEL_URL: &str = "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf";
const DEFAULT_MODEL_FILENAME: &str = "qwen2.5-0.5b-instruct-q4_k_m.gguf";

pub struct ModelDownloader {
    models_dir: PathBuf,
}

impl ModelDownloader {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let dir = PathBuf::from(format!("{}/Library/Application Support/com.promptos.app/models", home));
        std::fs::create_dir_all(&dir).ok();
        Self { models_dir: dir }
    }

    pub fn models_dir(&self) -> &Path {
        &self.models_dir
    }

    pub fn default_model_path(&self) -> PathBuf {
        self.models_dir.join(DEFAULT_MODEL_FILENAME)
    }

    pub fn is_model_downloaded(&self) -> bool {
        self.default_model_path().exists()
    }

    pub fn download_default_model(&self) -> Result<String, String> {
        let dest = self.default_model_path();
        if dest.exists() {
            info!("Download — default model already exists at {}", dest.display());
            return Ok(dest.to_string_lossy().to_string());
        }

        self.download_model(DEFAULT_MODEL_URL, &dest)
    }

    pub fn download_model(&self, url: &str, dest: &Path) -> Result<String, String> {
        info!("Download — url={}, dest={}, content_length=unknown", url, dest.display());
        println!("Downloading model (this may take a while)...");

        let response = reqwest::blocking::get(url)
            .map_err(|e| format!("Download failed: {}", e))?;

        let total = response.content_length().unwrap_or(0);
        let mut file = std::fs::File::create(dest)
            .map_err(|e| format!("Cannot create file: {}", e))?;

        let bytes = response.bytes().map_err(|e| format!("Download error: {}", e))?;
        std::io::Write::write_all(&mut file, &bytes)
            .map_err(|e| format!("Write error: {}", e))?;
        let downloaded = bytes.len() as u64;

        info!("Download — bytes_received={}, total={}, path={}", downloaded, total, dest.display());
        println!("Download complete: {}", format_size(downloaded));

        Ok(dest.to_string_lossy().to_string())
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
    fn test_downloader_creates_dir() {
        let dl = ModelDownloader::new();
        assert!(dl.models_dir().exists());
    }

    #[test]
    fn test_downloader_can_find_model() {
        let dl = ModelDownloader::new();
        let path = dl.default_model_path();
        println!("Default model path: {}", path.display());
    }
}
