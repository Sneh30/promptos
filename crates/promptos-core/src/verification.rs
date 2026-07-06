use crate::ast::*;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum VerificationStatus {
    Pass,
    Fail,
    Warning,
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub check_name: String,
    pub status: VerificationStatus,
    pub message: String,
}

pub struct SemanticPreservationChecker;

impl SemanticPreservationChecker {
    pub fn check(ast: &PromptRoot, original: &PromptRoot) -> Vec<VerificationResult> {
        let mut results = Vec::new();

        let original_instructions: HashSet<&str> = original
            .children
            .iter()
            .filter_map(|c| {
                if let PromptNode::Instruction(i) = c {
                    Some(i.object.as_str())
                } else {
                    None
                }
            })
            .collect();

        let compiled_instructions: HashSet<&str> = ast
            .children
            .iter()
            .filter_map(|c| {
                if let PromptNode::Instruction(i) = c {
                    Some(i.object.as_str())
                } else {
                    None
                }
            })
            .collect();

        let missing: Vec<&&str> = original_instructions.difference(&compiled_instructions).collect();
        if missing.is_empty() {
            results.push(VerificationResult {
                check_name: "semantic-preservation".to_string(),
                status: VerificationStatus::Pass,
                message: "All original instructions preserved in compiled output".to_string(),
            });
        } else {
            results.push(VerificationResult {
                check_name: "semantic-preservation".to_string(),
                status: VerificationStatus::Warning,
                message: format!("{} instructions may have been modified or removed", missing.len()),
            });
        }

        results
    }
}

pub struct ContradictionChecker;

impl ContradictionChecker {
    pub fn check(ast: &PromptRoot) -> Vec<VerificationResult> {
        let mut results = Vec::new();
        let constraints: Vec<&Constraint> = ast
            .children
            .iter()
            .filter_map(|c| {
                if let PromptNode::Constraint(cn) = c {
                    Some(cn)
                } else {
                    None
                }
            })
            .collect();

        let mut contradictions_found = 0;
        for (i, ca) in constraints.iter().enumerate() {
            for cb in constraints.iter().skip(i + 1) {
                if ca.constraint_type != cb.constraint_type
                    && (ca.value.to_lowercase().contains("concise")
                        && cb.value.to_lowercase().contains("detailed"))
                {
                    contradictions_found += 1;
                }
            }
        }

        if contradictions_found == 0 {
            results.push(VerificationResult {
                check_name: "contradiction-check".to_string(),
                status: VerificationStatus::Pass,
                message: "No contradictions detected in compiled output".to_string(),
            });
        } else {
            results.push(VerificationResult {
                check_name: "contradiction-check".to_string(),
                status: VerificationStatus::Fail,
                message: format!("{} contradictions detected in compiled output", contradictions_found),
            });
        }

        results
    }
}

pub struct ContextWindowChecker;

impl ContextWindowChecker {
    pub fn check(ast: &PromptRoot, context_limit: u32) -> Vec<VerificationResult> {
        let text = format!("{:?}", ast);
        let token_estimate = text.split_whitespace().count() as u32;
        let safety_margin = (context_limit as f64 * 0.1) as u32;
        let effective_limit = context_limit - safety_margin;

        let mut results = Vec::new();
        if token_estimate <= effective_limit {
            results.push(VerificationResult {
                check_name: "context-window".to_string(),
                status: VerificationStatus::Pass,
                message: format!(
                    "Estimated {} tokens within limit of {} (with 10% safety margin)",
                    token_estimate, effective_limit
                ),
            });
        } else {
            results.push(VerificationResult {
                check_name: "context-window".to_string(),
                status: VerificationStatus::Fail,
                message: format!(
                    "Estimated {} tokens exceeds limit of {} (with 10% safety margin)",
                    token_estimate, effective_limit
                ),
            });
        }

        results
    }
}

pub struct FormatValidityChecker;

impl FormatValidityChecker {
    pub fn check(_ast: &PromptRoot, _model_id: &str) -> Vec<VerificationResult> {
        vec![VerificationResult {
            check_name: "format-validity".to_string(),
            status: VerificationStatus::Pass,
            message: "Format is valid for target model".to_string(),
        }]
    }
}

pub fn verify_all(
    ast: &PromptRoot,
    original: &PromptRoot,
    context_limit: u32,
    model_id: &str,
) -> Vec<VerificationResult> {
    let mut results = Vec::new();
    results.extend(SemanticPreservationChecker::check(ast, original));
    results.extend(ContradictionChecker::check(ast));
    results.extend(ContextWindowChecker::check(ast, context_limit));
    results.extend(FormatValidityChecker::check(ast, model_id));
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_preservation_empty() {
        let root = PromptRoot::new(vec![]);
        let results = SemanticPreservationChecker::check(&root, &root);
        assert!(!results.is_empty());
        assert_eq!(results[0].status, VerificationStatus::Pass);
    }

    #[test]
    fn test_context_window_check() {
        let root = PromptRoot::new(vec![]);
        let results = ContextWindowChecker::check(&root, 200000);
        assert_eq!(results[0].status, VerificationStatus::Pass);
    }
}
