use crate::benchmark::{BenchmarkResult, BenchmarkSuiteResult, CategorySummary};
use log::{info, warn};

#[derive(Debug, Clone)]
pub struct RegressionReport {
    pub baseline_name: String,
    pub current_name: String,
    pub overall_regression: bool,
    pub category_regressions: Vec<CategoryRegression>,
    pub total_regressions: usize,
}

#[derive(Debug, Clone)]
pub struct CategoryRegression {
    pub category: String,
    pub baseline_avg_improvement: f32,
    pub current_avg_improvement: f32,
    pub delta_pct: f32,
    pub is_regression: bool,
}

pub struct RegressionChecker;

impl RegressionChecker {
    pub fn compare(
        baseline: &BenchmarkSuiteResult,
        current: &BenchmarkSuiteResult,
    ) -> RegressionReport {
        info!(
            "Regression check — baseline={}, current={}",
            baseline.suite_name, current.suite_name
        );
        let mut category_regressions = Vec::new();
        let mut total_regressions = 0usize;

        for base_cat in &baseline.category_summary {
            let current_cat = current
                .category_summary
                .iter()
                .find(|c| c.category == base_cat.category);

            if let Some(cur_cat) = current_cat {
                let delta = cur_cat.avg_improvement - base_cat.avg_improvement;
                let is_regression = delta < -2.0; // More than 2% degradation

                if is_regression {
                    total_regressions += 1;
                }

                category_regressions.push(CategoryRegression {
                    category: base_cat.category.clone(),
                    baseline_avg_improvement: base_cat.avg_improvement,
                    current_avg_improvement: cur_cat.avg_improvement,
                    delta_pct: delta,
                    is_regression,
                });
            }
        }

        if total_regressions > 0 {
            warn!(
                "Regression detected — {} categories regressed",
                total_regressions
            );
        } else {
            info!("Regression check — passed, no regressions detected");
        }
        RegressionReport {
            baseline_name: baseline.suite_name.clone(),
            current_name: current.suite_name.clone(),
            overall_regression: total_regressions > 0,
            category_regressions,
            total_regressions,
        }
    }

    pub fn is_blocked(report: &RegressionReport) -> bool {
        report.overall_regression
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_suite_result(name: &str, categories: Vec<(&str, f32)>) -> BenchmarkSuiteResult {
        BenchmarkSuiteResult {
            suite_name: name.to_string(),
            timestamp: 0,
            total_prompts: 0,
            passed: 0,
            failed: 0,
            average_improvement_pct: 0.0,
            results: vec![],
            category_summary: categories
                .into_iter()
                .map(|(cat, avg)| CategorySummary {
                    category: cat.to_string(),
                    count: 1,
                    avg_improvement: avg,
                    pass_rate: 1.0,
                })
                .collect(),
        }
    }

    #[test]
    fn test_regression_detection() {
        let baseline = make_suite_result("baseline", vec![("code", 15.0), ("writing", 20.0)]);

        let current = make_suite_result(
            "current",
            vec![
                ("code", 12.0), // -3.0 delta exceeds 2% threshold
                ("writing", 21.0),
            ],
        );

        let report = RegressionChecker::compare(&baseline, &current);
        assert!(report.overall_regression);
        assert_eq!(report.total_regressions, 1);
    }

    #[test]
    fn test_no_regression() {
        let baseline = make_suite_result("baseline", vec![("code", 15.0)]);

        let current = make_suite_result("current", vec![("code", 16.0)]);

        let report = RegressionChecker::compare(&baseline, &current);
        assert!(!report.overall_regression);
        assert_eq!(report.total_regressions, 0);
    }
}
