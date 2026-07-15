use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub timestamp: u64,
    pub user_prompt: String,
    pub compiled_prompt: String,
    pub target_model: String,
    pub mode: String,
    pub diagnostics: Vec<HistoryDiagnostic>,
    pub metrics: CompilationMetrics,
    pub diff: String,
    pub tags: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryDiagnostic {
    pub severity: String,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationMetrics {
    pub token_count_original: u32,
    pub token_count_compiled: u32,
    pub estimated_cost: f64,
    pub estimated_latency_ms: u64,
    pub quality_score: f32,
    pub hallucination_risk: f32,
    pub passes_applied: Vec<String>,
    pub compilation_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    pub max_entries: usize,
    pub storage_dir: String,
    pub compress: bool,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_entries: 100,
            storage_dir: String::new(),
            compress: true,
        }
    }
}

pub struct HistoryManager {
    config: HistoryConfig,
    index: Mutex<BTreeMap<u64, Uuid>>,
}

impl HistoryManager {
    pub fn new(config: HistoryConfig) -> Self {
        let storage_dir = if config.storage_dir.is_empty() {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!(
                "{}/Library/Application Support/com.promptos.app/history",
                home
            )
        } else {
            config.storage_dir.clone()
        };

        std::fs::create_dir_all(&storage_dir).ok();
        std::fs::create_dir_all(format!("{}/entries", storage_dir)).ok();

        Self {
            config: HistoryConfig {
                storage_dir,
                ..config
            },
            index: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn save(&self, entry: HistoryEntry) -> Result<(), String> {
        let path = format!("{}/entries/{}.msgpack", self.config.storage_dir, entry.id);
        info!(
            "History save — id={}, prompt_len={}, target_model={}",
            entry.id,
            entry.user_prompt.len(),
            entry.target_model
        );
        let data = rmp_serde::to_vec(&entry).map_err(|e| format!("Serialization error: {}", e))?;

        let data = if self.config.compress {
            zstd::encode_all(&data[..], 3).map_err(|e| format!("Compression error: {}", e))?
        } else {
            data
        };

        std::fs::write(&path, &data).map_err(|e| format!("Write error: {}", e))?;

        let mut index = self.index.lock().map_err(|e| e.to_string())?;
        index.insert(entry.timestamp, entry.id);

        while index.len() > self.config.max_entries {
            if let Some((oldest_ts, oldest_id)) = index.iter().next().map(|(k, v)| (*k, *v)) {
                index.remove(&oldest_ts);
                let old_path = format!("{}/entries/{}.msgpack", self.config.storage_dir, oldest_id);
                std::fs::remove_file(old_path).ok();
            }
        }
        drop(index);

        self.save_index()?;
        Ok(())
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<HistoryEntry>, String> {
        let path = format!("{}/entries/{}.msgpack", self.config.storage_dir, id);
        if !Path::new(&path).exists() {
            debug!("History get — not found: {}", id);
            return Ok(None);
        }
        debug!("History get — found: {}", id);

        let data = std::fs::read(&path).map_err(|e| format!("Read error: {}", e))?;
        let data = if self.config.compress {
            zstd::decode_all(&data[..]).map_err(|e| format!("Decompression error: {}", e))?
        } else {
            data
        };

        let entry: HistoryEntry =
            rmp_serde::from_slice(&data).map_err(|e| format!("Deserialization error: {}", e))?;

        Ok(Some(entry))
    }

    pub fn list(&self, limit: usize, offset: usize) -> Result<Vec<HistoryEntry>, String> {
        let ids: Vec<Uuid> = {
            let index = self.index.lock().map_err(|e| e.to_string())?;
            index
                .values()
                .rev()
                .skip(offset)
                .take(limit)
                .copied()
                .collect()
        };

        let mut results = Vec::new();
        for id in &ids {
            if let Ok(Some(entry)) = self.get(id) {
                results.push(entry);
            }
        }
        Ok(results)
    }

    pub fn delete(&self, id: &Uuid) -> Result<(), String> {
        info!("History delete — id={}", id);
        let path = format!("{}/entries/{}.msgpack", self.config.storage_dir, id);
        std::fs::remove_file(&path).ok();

        let mut index = self.index.lock().map_err(|e| e.to_string())?;
        index.retain(|_, v| v != id);
        drop(index);
        self.save_index()?;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), String> {
        let entries_dir = format!("{}/entries", self.config.storage_dir);
        if let Ok(dir) = std::fs::read_dir(&entries_dir) {
            for entry in dir.flatten() {
                std::fs::remove_file(entry.path()).ok();
            }
        }

        let mut index = self.index.lock().map_err(|e| e.to_string())?;
        index.clear();
        drop(index);
        self.save_index()?;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<Vec<HistoryEntry>, String> {
        info!("History search — query={}", query);
        let query_lower = query.to_lowercase();
        let entries = self.list(self.config.max_entries, 0)?;
        Ok(entries
            .into_iter()
            .filter(|e| {
                e.user_prompt.to_lowercase().contains(&query_lower)
                    || e.compiled_prompt.to_lowercase().contains(&query_lower)
                    || e.target_model.to_lowercase().contains(&query_lower)
                    || e.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect())
    }

    pub fn count(&self) -> Result<usize, String> {
        let index = self.index.lock().map_err(|e| e.to_string())?;
        Ok(index.len())
    }

    fn save_index(&self) -> Result<(), String> {
        let index_path = format!("{}/index.msgpack", self.config.storage_dir);
        let data = rmp_serde::to_vec(&*self.index.lock().map_err(|e| e.to_string())?)
            .map_err(|e| format!("Index serialization error: {}", e))?;
        std::fs::write(&index_path, &data).map_err(|e| format!("Index write error: {}", e))
    }

    #[allow(dead_code)]
    fn load_index(&self) -> Result<(), String> {
        let index_path = format!("{}/index.msgpack", self.config.storage_dir);
        if Path::new(&index_path).exists() {
            let data =
                std::fs::read(&index_path).map_err(|e| format!("Index read error: {}", e))?;
            let index: BTreeMap<u64, Uuid> = rmp_serde::from_slice(&data)
                .map_err(|e| format!("Index deserialization error: {}", e))?;
            *self.index.lock().map_err(|e| e.to_string())? = index;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> HistoryManager {
        let dir = std::env::temp_dir().join(format!("promptos-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).ok();
        let config = HistoryConfig {
            storage_dir: dir.to_string_lossy().to_string(),
            max_entries: 10,
            compress: true,
        };
        HistoryManager::new(config)
    }

    #[test]
    fn test_save_and_get() {
        let manager = create_test_manager();
        let id = Uuid::new_v4();
        let entry = HistoryEntry {
            id,
            timestamp: 1000,
            user_prompt: "Test prompt".to_string(),
            compiled_prompt: "Compiled prompt".to_string(),
            target_model: "claude-3.5-sonnet".to_string(),
            mode: "balanced".to_string(),
            diagnostics: vec![],
            metrics: CompilationMetrics {
                token_count_original: 10,
                token_count_compiled: 8,
                estimated_cost: 0.001,
                estimated_latency_ms: 100,
                quality_score: 8.5,
                hallucination_risk: 0.05,
                passes_applied: vec![],
                compilation_time_ms: 50,
            },
            diff: String::new(),
            tags: vec![],
            metadata: std::collections::HashMap::new(),
        };

        manager.save(entry.clone()).unwrap();
        let retrieved = manager.get(&id).unwrap().unwrap();
        assert_eq!(retrieved.user_prompt, "Test prompt");
        assert_eq!(retrieved.target_model, "claude-3.5-sonnet");
    }

    #[test]
    fn test_list_entries() {
        let manager = create_test_manager();
        for i in 0..5 {
            let entry = HistoryEntry {
                id: Uuid::new_v4(),
                timestamp: 1000 + i,
                user_prompt: format!("Prompt {}", i),
                compiled_prompt: format!("Compiled {}", i),
                target_model: "test".to_string(),
                mode: "balanced".to_string(),
                diagnostics: vec![],
                metrics: CompilationMetrics {
                    token_count_original: 1,
                    token_count_compiled: 1,
                    estimated_cost: 0.0,
                    estimated_latency_ms: 0,
                    quality_score: 5.0,
                    hallucination_risk: 0.0,
                    passes_applied: vec![],
                    compilation_time_ms: 0,
                },
                diff: String::new(),
                tags: vec![],
                metadata: std::collections::HashMap::new(),
            };
            manager.save(entry).unwrap();
        }

        let entries = manager.list(10, 0).unwrap();
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn test_search() {
        let manager = create_test_manager();
        let entry = HistoryEntry {
            id: Uuid::new_v4(),
            timestamp: 1000,
            user_prompt: "Write a poem about AI".to_string(),
            compiled_prompt: "Compiled poem".to_string(),
            target_model: "claude-3.5-sonnet".to_string(),
            mode: "balanced".to_string(),
            diagnostics: vec![],
            metrics: CompilationMetrics {
                token_count_original: 5,
                token_count_compiled: 3,
                estimated_cost: 0.0,
                estimated_latency_ms: 0,
                quality_score: 7.0,
                hallucination_risk: 0.0,
                passes_applied: vec![],
                compilation_time_ms: 0,
            },
            diff: String::new(),
            tags: vec!["poem".to_string()],
            metadata: std::collections::HashMap::new(),
        };
        manager.save(entry).unwrap();

        let results = manager.search("poem").unwrap();
        assert!(!results.is_empty());
        assert!(results[0].user_prompt.contains("poem"));
    }

    #[test]
    fn test_max_entries() {
        let manager = create_test_manager();
        for i in 0..15 {
            let entry = HistoryEntry {
                id: Uuid::new_v4(),
                timestamp: 1000 + i,
                user_prompt: format!("Prompt {}", i),
                compiled_prompt: String::new(),
                target_model: "test".to_string(),
                mode: "balanced".to_string(),
                diagnostics: vec![],
                metrics: CompilationMetrics {
                    token_count_original: 1,
                    token_count_compiled: 1,
                    estimated_cost: 0.0,
                    estimated_latency_ms: 0,
                    quality_score: 5.0,
                    hallucination_risk: 0.0,
                    passes_applied: vec![],
                    compilation_time_ms: 0,
                },
                diff: String::new(),
                tags: vec![],
                metadata: std::collections::HashMap::new(),
            };
            manager.save(entry).unwrap();
        }
        assert_eq!(manager.count().unwrap(), 10);
    }
}
