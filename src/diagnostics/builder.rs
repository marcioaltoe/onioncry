use crate::{ImportEdge, Severity, Violation};
use std::path::Path;

pub(super) fn base_violation(
    rule: &str,
    severity: Severity,
    file: &Path,
    message: impl Into<String>,
) -> Violation {
    Violation {
        rule: rule.to_string(),
        severity: severity.as_str().to_string(),
        message: message.into(),
        file: file.display().to_string(),
        import_specifier: None,
        package_name: None,
        line: None,
        column: None,
        from_layer: None,
        to_layer: None,
        from_context: None,
        to_context: None,
        target_file: None,
        cycle_path: None,
        suggestion: None,
        matched_layers: None,
        matched_contexts: None,
    }
}

pub(super) fn import_violation(
    rule: &str,
    severity: Severity,
    edge: &ImportEdge,
    message: impl Into<String>,
) -> Violation {
    let mut violation = base_violation(rule, severity, &edge.source, message);
    violation.import_specifier = Some(edge.specifier.clone());
    violation.line = Some(edge.line);
    violation.column = Some(edge.column);
    violation
}
