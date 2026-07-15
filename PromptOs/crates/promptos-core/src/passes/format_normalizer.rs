use super::*;
use async_trait::async_trait;
use log::info;

pub struct FormatNormalizationPass;

const FORMAT_INDICATORS: &[(&str, &str)] = &[
    ("json", "JSON format"),
    ("xml", "XML format"),
    ("markdown", "Markdown format"),
    ("csv", "CSV format"),
    ("yaml", "YAML format"),
    ("html", "HTML format"),
    ("table", "table format"),
];

#[async_trait]
impl OptimizationPass for FormatNormalizationPass {
    fn name(&self) -> &str {
        "format-normalization"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn should_run(&self, _mode: CompilationMode, _config: &UserConfig) -> bool {
        true
    }

    async fn run(&self, ast: &mut PromptRoot, ctx: &PassContext) -> Result<PassResult, PassError> {
        info!(
            "Pass [format_normalizer] — entering, target_model={}",
            ctx.target_model
        );
        let mut normalized = 0;
        let preferred_format = self.get_preferred_format(&ctx.target_model);

        for child in &mut ast.children {
            if let PromptNode::FormatSpec(spec) = child {
                let lower = spec.format_type.to_lowercase();
                for (indicator, _) in FORMAT_INDICATORS {
                    if lower.contains(indicator) && spec.format_type != preferred_format {
                        spec.format_type = preferred_format.to_string();
                        normalized += 1;
                        break;
                    }
                }
            }
        }

        info!(
            "Pass [format_normalizer] — exit, normalized={}, preferred={}",
            normalized, preferred_format
        );
        Ok(PassResult {
            pass_name: self.name().to_string(),
            tokens_saved: 0,
            applied: normalized > 0,
            description: format!(
                "Normalized {} format specifications to {} preferred format",
                normalized, ctx.target_model
            ),
        })
    }

    fn verify(&self, _ast: &PromptRoot, _original: &PromptRoot) -> Result<(), VerificationFailure> {
        Ok(())
    }
}

impl FormatNormalizationPass {
    fn get_preferred_format(&self, model_id: &str) -> String {
        if model_id.contains("claude") {
            "XML format".to_string()
        } else if model_id.contains("gpt") || model_id.contains("openai") {
            "Markdown format".to_string()
        } else if model_id.contains("gemini") {
            "JSON format".to_string()
        } else {
            "Markdown format".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_format_normalization_claude() {
        let mut ast = PromptRoot::new(vec![PromptNode::FormatSpec(FormatSpec {
            format_type: "json".to_string(),
            detail: "Output as JSON".to_string(),
            span: crate::ast::SourceSpan::new(Position::new(1, 1), Position::new(1, 15)),
        })]);
        let pass = FormatNormalizationPass;
        let ctx = PassContext {
            model_profile: None,
            annotations: Annotations::default(),
            config: UserConfig::default(),
            target_model: "claude-3.5-sonnet".to_string(),
        };
        let result = pass.run(&mut ast, &ctx).await.unwrap();
        assert!(result.applied);
        if let PromptNode::FormatSpec(spec) = &ast.children[0] {
            assert_eq!(spec.format_type, "XML format");
        }
    }
}
