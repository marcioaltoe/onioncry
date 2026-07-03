use crate::rules::catalog::{RULE_FEATURE_ENVY, RULE_NO_CONCRETE_DEPENDENCY, RULE_SHOTGUN_SURGERY};
use crate::{ImportEdge, Severity, Violation};
use std::path::Path;

impl Violation {
    pub(crate) fn concrete_dependency(
        edge: &ImportEdge,
        target: Option<&Path>,
        from_layer: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_CONCRETE_DEPENDENCY.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not depend on concrete detail through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: target.map(|target| target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "depend on an abstraction owned by the core layer and bind the concrete detail outside"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn feature_envy(
        file: &Path,
        from_context: &str,
        to_context: &str,
        import_count: usize,
        own_context_count: usize,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_ENVY.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_context} file imports {import_count} dependencies from {to_context} and {own_context_count} from its own context"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: Some(from_context.to_string()),
            to_context: Some(to_context.to_string()),
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "move the behavior closer to the context it uses or depend on a smaller public contract"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn shotgun_surgery(
        file: &Path,
        commit_count: usize,
        related_file_count: usize,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_SHOTGUN_SURGERY.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "file changed in {commit_count} commits with {related_file_count} recurring companion files"
            ),
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
            suggestion: Some(
                "look for scattered responsibilities and extract a boundary that changes together"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }
}
