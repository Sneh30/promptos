use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkPrompt {
    pub id: String,
    pub category: String,
    pub prompt: String,
    pub expected_output: Option<String>,
    pub scoring_rubric: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub prompt_id: String,
    pub category: String,
    pub raw_score: f32,
    pub compiled_score: f32,
    pub improvement_pct: f32,
    pub raw_tokens: u32,
    pub compiled_tokens: u32,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuiteResult {
    pub suite_name: String,
    pub timestamp: u64,
    pub total_prompts: usize,
    pub passed: usize,
    pub failed: usize,
    pub average_improvement_pct: f32,
    pub results: Vec<BenchmarkResult>,
    pub category_summary: Vec<CategorySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category: String,
    pub count: usize,
    pub avg_improvement: f32,
    pub pass_rate: f32,
}

pub struct BenchmarkSuite {
    #[allow(dead_code)]
    name: String,
    prompts: Vec<BenchmarkPrompt>,
}

impl BenchmarkSuite {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            prompts: Vec::new(),
        }
    }

    pub fn add_prompt(&mut self, prompt: BenchmarkPrompt) {
        debug!(
            "Benchmark add_prompt — id={}, category={}",
            prompt.id, prompt.category
        );
        self.prompts.push(prompt);
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<usize, String> {
        info!("Benchmark load_from_file — path={}", path);
        if !Path::new(path).exists() {
            return Err(format!("Benchmark file not found: {}", path));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read benchmark file: {}", e))?;

        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let prompts: Vec<BenchmarkPrompt> = match ext {
            "json" => {
                serde_json::from_str(&content).map_err(|e| format!("JSON parse error: {}", e))?
            }
            "toml" => {
                let single: BenchmarkPrompt =
                    toml::from_str(&content).map_err(|e| format!("TOML parse error: {}", e))?;
                vec![single]
            }
            _ => return Err(format!("Unsupported file format: {}", ext)),
        };

        let count = prompts.len();
        self.prompts.extend(prompts);
        Ok(count)
    }

    pub fn prompts(&self) -> &[BenchmarkPrompt] {
        &self.prompts
    }

    pub fn len(&self) -> usize {
        self.prompts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.prompts.is_empty()
    }

    pub fn by_category(&self, category: &str) -> Vec<&BenchmarkPrompt> {
        self.prompts
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }

    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.prompts.iter().map(|p| p.category.clone()).collect();
        cats.sort();
        cats.dedup();
        cats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite_empty() {
        let suite = BenchmarkSuite::new("test");
        assert!(suite.is_empty());
        assert_eq!(suite.len(), 0);
    }

    #[test]
    fn test_benchmark_suite_add_prompt() {
        let mut suite = BenchmarkSuite::new("test");
        suite.add_prompt(BenchmarkPrompt {
            id: "test-001".to_string(),
            category: "code-generation".to_string(),
            prompt: "Write a function".to_string(),
            expected_output: None,
            scoring_rubric: None,
            tags: vec![],
        });
        assert_eq!(suite.len(), 1);
    }

    #[test]
    fn test_benchmark_categories() {
        let mut suite = BenchmarkSuite::new("test");
        suite.add_prompt(BenchmarkPrompt {
            id: "1".to_string(),
            category: "code".to_string(),
            prompt: "Write code".to_string(),
            expected_output: None,
            scoring_rubric: None,
            tags: vec![],
        });
        suite.add_prompt(BenchmarkPrompt {
            id: "2".to_string(),
            category: "writing".to_string(),
            prompt: "Write text".to_string(),
            expected_output: None,
            scoring_rubric: None,
            tags: vec![],
        });
        let cats = suite.categories();
        assert_eq!(cats.len(), 2);
    }
}
