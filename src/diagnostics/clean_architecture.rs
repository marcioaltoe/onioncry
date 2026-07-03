use crate::*;
use std::collections::HashSet;
use std::path::Path;

impl Violation {
    pub(crate) fn unclassified_file(file: &Path, severity: Severity) -> Self {
        Self {
            rule: RULE_UNCLASSIFIED_FILE.to_string(),
            severity: severity.as_str().to_string(),
            message: "file is not classified by any configured architectural layer".to_string(),
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
                "add a matching layers.*.patterns entry or exclude the file".to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn ambiguous_layer(
        file: &Path,
        matched_layers: Vec<String>,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_AMBIGUOUS_LAYER.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "file matches multiple architectural layers: {}",
                matched_layers.join(", ")
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
            suggestion: Some("make layer patterns mutually exclusive".to_string()),
            matched_layers: Some(matched_layers),
            matched_contexts: None,
        }
    }

    pub(crate) fn layer_leak(
        edge: &ImportEdge,
        target: &Path,
        from_layer: &str,
        to_layer: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_LAYER_LEAK.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not import {to_layer} through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: Some(to_layer.to_string()),
            from_context: None,
            to_context: None,
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(format!(
                "add {to_layer:?} to layers.{from_layer}.mayImport only if this dependency is intentional"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn forbidden_external_package(
        edge: &ImportEdge,
        from_layer: &str,
        package_name: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not import external package {package_name} through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: Some(package_name.to_string()),
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "add {package_name:?} to the cleanarch/no-forbidden-imports allowlist for {from_layer} only if this package is domain-safe for that layer"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn framework_in_core(
        edge: &ImportEdge,
        from_layer: &str,
        package_name: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_FRAMEWORK_IN_CORE.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not depend on framework package {package_name} through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: Some(package_name.to_string()),
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "depend on a core-owned port and move framework code to an outer layer".to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn outer_data_format_in_core(
        edge: &ImportEdge,
        target: &Path,
        from_layer: &str,
        to_layer: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_OUTER_DATA_FORMAT_IN_CORE.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not import {to_layer} data format through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: Some(to_layer.to_string()),
            from_context: None,
            to_context: None,
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "define a core-owned type or mapper instead of importing outer data formats"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn ambiguous_context(
        file: &Path,
        matched_contexts: Vec<String>,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_AMBIGUOUS_CONTEXT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "file matches multiple architectural contexts: {}",
                matched_contexts.join(", ")
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
            suggestion: Some("make context patterns mutually exclusive".to_string()),
            matched_layers: None,
            matched_contexts: Some(matched_contexts),
        }
    }

    pub(crate) fn cross_context_internal_import(
        edge: &ImportEdge,
        target: &Path,
        from_context: &str,
        to_context: &str,
        severity: Severity,
        public_surface_segments: &HashSet<String>,
    ) -> Self {
        let mut segments = public_surface_segments.iter().cloned().collect::<Vec<_>>();
        segments.sort();
        Self {
            rule: RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_context} may not import {to_context} internal details through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: Some(from_context.to_string()),
            to_context: Some(to_context.to_string()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(format!(
                "import from the {to_context} public surface segment instead{}",
                if segments.is_empty() {
                    String::new()
                } else {
                    format!(": {}", segments.join(", "))
                }
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn public_surface_internal_reexport(
        edge: &ImportEdge,
        target: &Path,
        context: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{context} public surface may not re-export internal detail through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: Some(context.to_string()),
            to_context: Some(context.to_string()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "move the contract into the public surface or stop re-exporting it".to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn context_cycle(
        edge: &ImportEdge,
        target: &Path,
        context_path: &[String],
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_CONTEXT_CYCLE.to_string(),
            severity: severity.as_str().to_string(),
            message: format!("context dependency cycle: {}", context_path.join(" -> ")),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: context_path.first().cloned(),
            to_context: context_path.get(1).cloned(),
            target_file: Some(target.display().to_string()),
            cycle_path: Some(context_path.to_vec()),
            suggestion: Some(
                "extract a public contract or shared kernel so context dependencies point one way"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn unowned_schema_import(
        edge: &ImportEdge,
        target: &Path,
        from_context: &str,
        to_context: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_UNOWNED_SCHEMA_IMPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_context} may not import {to_context} owned schema through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: Some(from_context.to_string()),
            to_context: Some(to_context.to_string()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "depend on the owning context contract instead of importing its storage schema"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn clean_artifact_placement(
        file: &Path,
        severity: Severity,
        role: &str,
        expected_layer: &str,
        expected_boundary: &str,
    ) -> Self {
        Self {
            rule: RULE_CLEAN_ARTIFACT_PLACEMENT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "cleanArchitecture artifact {role:?} should live in the {expected_layer} boundary {expected_boundary}"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: Some(expected_layer.to_string()),
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "move this {role} artifact to {expected_boundary} or turn cleanarch/artifact-placement off with an override while migrating"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }
}
