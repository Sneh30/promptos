use crate::ast::*;

pub struct DiagnosticBuilder {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBuilder {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn error(&mut self, code: &str, message: &str, span: Option<SourceSpan>, recommendation: Option<&str>) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            message: message.to_string(),
            span,
            diagnostic_code: code.to_string(),
            recommendation: recommendation.map(|s| s.to_string()),
        });
    }

    pub fn warning(&mut self, code: &str, message: &str, span: Option<SourceSpan>, recommendation: Option<&str>) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Warning,
            message: message.to_string(),
            span,
            diagnostic_code: code.to_string(),
            recommendation: recommendation.map(|s| s.to_string()),
        });
    }

    pub fn suggestion(&mut self, code: &str, message: &str, span: Option<SourceSpan>, recommendation: Option<&str>) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Suggestion,
            message: message.to_string(),
            span,
            diagnostic_code: code.to_string(),
            recommendation: recommendation.map(|s| s.to_string()),
        });
    }

    pub fn info(&mut self, code: &str, message: &str, span: Option<SourceSpan>, recommendation: Option<&str>) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Info,
            message: message.to_string(),
            span,
            diagnostic_code: code.to_string(),
            recommendation: recommendation.map(|s| s.to_string()),
        });
    }

    pub fn from_annotations(annotations: &Annotations) -> Self {
        let mut builder = Self::new();
        for ambiguity in &annotations.detected_ambiguities {
            builder.warning(
                "AMB-001",
                &format!("Ambiguous term: '{}'", ambiguity.text),
                Some(ambiguity.span),
                ambiguity.recommended_resolution.as_deref(),
            );
        }
        for contradiction in &annotations.detected_contradictions {
            builder.error(
                "CON-001",
                &contradiction.description,
                Some(contradiction.constraint_a),
                Some("Review and resolve conflicting constraints"),
            );
        }
        for gap in &annotations.detected_context_gaps {
            builder.warning(
                "GAP-001",
                &format!("Context gap: {}", gap.description),
                None,
                gap.suggested_addition.as_deref(),
            );
        }
        builder
    }

    pub fn build(&self) -> Vec<Diagnostic> {
        self.diagnostics.clone()
    }

    pub fn extend(&mut self, other: Vec<Diagnostic>) {
        self.diagnostics.extend(other);
    }

    pub fn count_by_severity(&self) -> (usize, usize, usize, usize) {
        let mut errors = 0;
        let mut warnings = 0;
        let mut suggestions = 0;
        let mut infos = 0;

        for d in &self.diagnostics {
            match d.severity {
                DiagnosticSeverity::Error => errors += 1,
                DiagnosticSeverity::Warning => warnings += 1,
                DiagnosticSeverity::Suggestion => suggestions += 1,
                DiagnosticSeverity::Info => infos += 1,
            }
        }

        (errors, warnings, suggestions, infos)
    }

    pub fn summary_string(&self) -> String {
        let (errors, warnings, suggestions, infos) = self.count_by_severity();
        let mut parts = Vec::new();
        if errors > 0 { parts.push(format!("{} errors", errors)); }
        if warnings > 0 { parts.push(format!("{} warnings", warnings)); }
        if suggestions > 0 { parts.push(format!("{} suggestions", suggestions)); }
        if infos > 0 { parts.push(format!("{} info", infos)); }
        if parts.is_empty() { return "No diagnostics".to_string(); }
        parts.join(", ")
    }
}

impl Default for DiagnosticBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_builder_empty() {
        let builder = DiagnosticBuilder::new();
        assert_eq!(builder.build().len(), 0);
    }

    #[test]
    fn test_diagnostic_builder_with_entries() {
        let mut builder = DiagnosticBuilder::new();
        builder.error("ERR-001", "Test error", None, Some("Fix it"));
        builder.warning("WARN-001", "Test warning", None, None);
        assert_eq!(builder.build().len(), 2);
    }

    #[test]
    fn test_summary_string() {
        let mut builder = DiagnosticBuilder::new();
        builder.error("E1", "Error", None, None);
        builder.warning("W1", "Warning", None, None);
        let summary = builder.summary_string();
        assert!(summary.contains("1 errors"));
        assert!(summary.contains("1 warnings"));
    }

    #[test]
    fn test_count_by_severity() {
        let mut builder = DiagnosticBuilder::new();
        builder.error("E1", "Error", None, None);
        builder.warning("W1", "Warning", None, None);
        builder.warning("W2", "Warning 2", None, None);
        builder.suggestion("S1", "Suggestion", None, None);
        let (errors, warnings, suggestions, infos) = builder.count_by_severity();
        assert_eq!(errors, 1);
        assert_eq!(warnings, 2);
        assert_eq!(suggestions, 1);
        assert_eq!(infos, 0);
    }
}
