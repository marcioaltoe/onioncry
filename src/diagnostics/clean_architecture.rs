use super::builder::{base_violation, import_violation};
use crate::rules::catalog::{
    RULE_AMBIGUOUS_CONTEXT, RULE_AMBIGUOUS_LAYER, RULE_CLEAN_ARTIFACT_PLACEMENT,
    RULE_NO_CONTEXT_CYCLE, RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT, RULE_NO_FORBIDDEN_IMPORTS,
    RULE_NO_FRAMEWORK_IN_CORE, RULE_NO_LAYER_LEAK, RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
    RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT, RULE_NO_UNOWNED_SCHEMA_IMPORT,
    RULE_UNCLASSIFIED_FILE,
};
use crate::{ImportEdge, Severity, Violation};
use std::collections::HashSet;
use std::path::Path;

impl Violation {
    pub(crate) fn unclassified_file(file: &Path, severity: Severity) -> Self {
        let mut violation = base_violation(
            RULE_UNCLASSIFIED_FILE,
            severity,
            file,
            "file is not classified by any configured architectural layer",
        );
        violation.suggestion =
            Some("add a matching layers.*.patterns entry or exclude the file".to_string());

        violation
    }

    pub(crate) fn ambiguous_layer(
        file: &Path,
        matched_layers: Vec<String>,
        severity: Severity,
    ) -> Self {
        let mut violation = base_violation(
            RULE_AMBIGUOUS_LAYER,
            severity,
            file,
            format!(
                "file matches multiple architectural layers: {}",
                matched_layers.join(", ")
            ),
        );
        violation.suggestion = Some("make layer patterns mutually exclusive".to_string());
        violation.matched_layers = Some(matched_layers);

        violation
    }

    pub(crate) fn layer_leak(
        edge: &ImportEdge,
        target: &Path,
        from_layer: &str,
        to_layer: &str,
        severity: Severity,
    ) -> Self {
        let mut violation = import_violation(
            RULE_NO_LAYER_LEAK,
            severity,
            edge,
            format!(
                "{from_layer} may not import {to_layer} through {}",
                edge.specifier
            ),
        );
        violation.from_layer = Some(from_layer.to_string());
        violation.to_layer = Some(to_layer.to_string());
        violation.target_file = Some(target.display().to_string());
        violation.suggestion = Some(format!(
            "add {to_layer:?} to layers.{from_layer}.mayImport only if this dependency is intentional"
        ));
        violation
    }

    pub(crate) fn forbidden_external_package(
        edge: &ImportEdge,
        from_layer: &str,
        package_name: &str,
        severity: Severity,
    ) -> Self {
        let mut violation = import_violation(
            RULE_NO_FORBIDDEN_IMPORTS,
            severity,
            edge,
            format!(
                "{from_layer} may not import external package {package_name} through {}",
                edge.specifier
            ),
        );
        violation.package_name = Some(package_name.to_string());
        violation.from_layer = Some(from_layer.to_string());
        violation.suggestion = Some(format!(
            "add {package_name:?} to the cleanarch/no-forbidden-imports allowlist for {from_layer} only if this package is domain-safe for that layer"
        ));
        violation
    }

    pub(crate) fn framework_in_core(
        edge: &ImportEdge,
        from_layer: &str,
        package_name: &str,
        severity: Severity,
    ) -> Self {
        let mut violation = import_violation(
            RULE_NO_FRAMEWORK_IN_CORE,
            severity,
            edge,
            format!(
                "{from_layer} may not depend on framework package {package_name} through {}",
                edge.specifier
            ),
        );
        violation.package_name = Some(package_name.to_string());
        violation.from_layer = Some(from_layer.to_string());
        violation.suggestion = Some(
            "depend on a core-owned port and move framework code to an outer layer".to_string(),
        );
        violation
    }

    pub(crate) fn outer_data_format_in_core(
        edge: &ImportEdge,
        target: &Path,
        from_layer: &str,
        to_layer: &str,
        severity: Severity,
    ) -> Self {
        let mut violation = import_violation(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            severity,
            edge,
            format!(
                "{from_layer} may not import {to_layer} data format through {}",
                edge.specifier
            ),
        );
        violation.from_layer = Some(from_layer.to_string());
        violation.to_layer = Some(to_layer.to_string());
        violation.target_file = Some(target.display().to_string());
        violation.suggestion = Some(
            "define a core-owned type or mapper instead of importing outer data formats"
                .to_string(),
        );
        violation
    }

    pub(crate) fn ambiguous_context(
        file: &Path,
        matched_contexts: Vec<String>,
        severity: Severity,
    ) -> Self {
        let mut violation = base_violation(
            RULE_AMBIGUOUS_CONTEXT,
            severity,
            file,
            format!(
                "file matches multiple architectural contexts: {}",
                matched_contexts.join(", ")
            ),
        );
        violation.suggestion = Some("make context patterns mutually exclusive".to_string());
        violation.matched_contexts = Some(matched_contexts);

        violation
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
        let mut violation = import_violation(
            RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT,
            severity,
            edge,
            format!(
                "{from_context} may not import {to_context} internal details through {}",
                edge.specifier
            ),
        );
        violation.from_context = Some(from_context.to_string());
        violation.to_context = Some(to_context.to_string());
        violation.target_file = Some(target.display().to_string());
        violation.suggestion = Some(format!(
            "import from the {to_context} public surface segment instead{}",
            if segments.is_empty() {
                String::new()
            } else {
                format!(": {}", segments.join(", "))
            }
        ));
        violation
    }

    pub(crate) fn public_surface_internal_reexport(
        edge: &ImportEdge,
        target: &Path,
        context: &str,
        severity: Severity,
    ) -> Self {
        let mut violation = import_violation(
            RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT,
            severity,
            edge,
            format!(
                "{context} public surface may not re-export internal detail through {}",
                edge.specifier
            ),
        );
        violation.from_context = Some(context.to_string());
        violation.to_context = Some(context.to_string());
        violation.target_file = Some(target.display().to_string());
        violation.suggestion =
            Some("move the contract into the public surface or stop re-exporting it".to_string());
        violation
    }

    pub(crate) fn context_cycle(
        edge: &ImportEdge,
        target: &Path,
        context_path: &[String],
        severity: Severity,
    ) -> Self {
        let mut violation = import_violation(
            RULE_NO_CONTEXT_CYCLE,
            severity,
            edge,
            format!("context dependency cycle: {}", context_path.join(" -> ")),
        );
        violation.from_context = context_path.first().cloned();
        violation.to_context = context_path.get(1).cloned();
        violation.target_file = Some(target.display().to_string());
        violation.cycle_path = Some(context_path.to_vec());
        violation.suggestion = Some(
            "extract a public contract or shared kernel so context dependencies point one way"
                .to_string(),
        );
        violation
    }

    pub(crate) fn unowned_schema_import(
        edge: &ImportEdge,
        target: &Path,
        from_context: &str,
        to_context: &str,
        severity: Severity,
    ) -> Self {
        let mut violation = import_violation(
            RULE_NO_UNOWNED_SCHEMA_IMPORT,
            severity,
            edge,
            format!(
                "{from_context} may not import {to_context} owned schema through {}",
                edge.specifier
            ),
        );
        violation.from_context = Some(from_context.to_string());
        violation.to_context = Some(to_context.to_string());
        violation.target_file = Some(target.display().to_string());
        violation.suggestion = Some(
            "depend on the owning context contract instead of importing its storage schema"
                .to_string(),
        );
        violation
    }

    pub(crate) fn clean_artifact_placement(
        file: &Path,
        severity: Severity,
        role: &str,
        expected_layer: &str,
        expected_boundary: &str,
    ) -> Self {
        let mut violation = base_violation(
            RULE_CLEAN_ARTIFACT_PLACEMENT,
            severity,
            file,
            format!(
                "cleanArchitecture artifact {role:?} should live in the {expected_layer} boundary {expected_boundary}"
            ),
        );
        violation.to_layer = Some(expected_layer.to_string());
        violation.suggestion = Some(format!(
            "move this {role} artifact to {expected_boundary} or turn cleanarch/artifact-placement off with an override while migrating"
        ));

        violation
    }
}
