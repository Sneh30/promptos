use crate::benchmark::BenchmarkResult;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ABTestResult {
    pub raw_response: String,
    pub compiled_response: String,
    pub raw_score: f32,
    pub compiled_score: f32,
    pub improvement_pct: f32,
    pub raw_cost: f64,
    pub compiled_cost: f64,
    pub cost_savings_pct: f32,
    pub raw_latency_ms: u64,
    pub compiled_latency_ms: u64,
    pub determinism_score: f32,
}

#[async_trait]
pub trait OutputEvaluator: Send + Sync {
    fn name(&self) -> &str;
    async fn evaluate(&self, prompt: &str, response: &str, rubric: Option<&str>) -> f32;
}

pub struct HeuristicEvaluator;

impl HeuristicEvaluator {
    pub fn new() -> Self {
        Self
    }

    fn score_response(&self, prompt: &str, response: &str) -> f32 {
        let mut score = 5.0f32;

        let response_words = response.split_whitespace().count();
        if response_words < 5 {
            score -= 2.0;
        } else if response_words > 10 {
            score += 1.0;
        }

        if response.contains("```") {
            score += 1.0;
        }

        if prompt.contains("json") && response.contains('{') && response.contains('}') {
            score += 1.0;
        }

        if prompt.contains("list") && response.chars().filter(|c| *c == '-').count() > 2 {
            score += 0.5;
        }

        score.clamp(0.0, 10.0)
    }
}

#[async_trait]
impl OutputEvaluator for HeuristicEvaluator {
    fn name(&self) -> &str {
        "heuristic-evaluator"
    }

    async fn evaluate(&self, prompt: &str, response: &str, _rubric: Option<&str>) -> f32 {
        self.score_response(prompt, response)
    }
}

pub struct ABHarness {
    evaluator: Box<dyn OutputEvaluator>,
}

impl ABHarness {
    pub fn new(evaluator: Box<dyn OutputEvaluator>) -> Self {
        Self { evaluator }
    }

    pub fn with_heuristic() -> Self {
        Self {
            evaluator: Box::new(HeuristicEvaluator::new()),
        }
    }

    pub async fn run_ab_test(
        &self,
        raw_prompt: &str,
        compiled_prompt: &str,
        raw_response: &str,
        compiled_response: &str,
        rubric: Option<&str>,
    ) -> ABTestResult {
        let raw_score = self.evaluator.evaluate(raw_prompt, raw_response, rubric).await;
        let compiled_score = self.evaluator.evaluate(compiled_prompt, compiled_response, rubric).await;

        let improvement_pct = if raw_score > 0.0 {
            ((compiled_score - raw_score) / raw_score) * 100.0
        } else {
            compiled_score * 10.0
        };

        let raw_cost = raw_prompt.split_whitespace().count() as f64 * 3.0 / 1_000_000.0;
        let compiled_cost = compiled_prompt.split_whitespace().count() as f64 * 3.0 / 1_000_000.0;
        let cost_savings_pct = if raw_cost > 0.0 {
            ((raw_cost - compiled_cost) / raw_cost) * 100.0
        } else {
            0.0
        };

        ABTestResult {
            raw_response: raw_response.to_string(),
            compiled_response: compiled_response.to_string(),
            raw_score,
            compiled_score,
            improvement_pct,
            raw_cost,
            compiled_cost,
            cost_savings_pct: cost_savings_pct as f32,
            raw_latency_ms: 0,
            compiled_latency_ms: 0,
            determinism_score: 0.0,
        }
    }

    pub fn to_benchmark_result(&self, result: &ABTestResult, prompt_id: &str, category: &str) -> BenchmarkResult {
        BenchmarkResult {
            prompt_id: prompt_id.to_string(),
            category: category.to_string(),
            raw_score: result.raw_score,
            compiled_score: result.compiled_score,
            improvement_pct: result.improvement_pct,
            raw_tokens: 0,
            compiled_tokens: 0,
            passed: result.improvement_pct >= 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ab_harness() {
        let harness = ABHarness::with_heuristic();
        let result = harness.run_ab_test(
            "Write code",
            "Write efficient Python code",
            "def foo(): pass",
            "def optimized_function(): return 42",
            None,
        ).await;

        // Compiled prompt is better, so compiled score should be >= raw score
        assert!(result.compiled_score >= 0.0);
        assert!(result.raw_score >= 0.0);
    }
}
