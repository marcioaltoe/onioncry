use super::builder::base_violation;
use crate::rules::catalog::{
    RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT, RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
    RULE_FEATURE_SYSTEM_LAYOUT, RULE_FEATURE_SYSTEM_PUBLIC_API, RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
};
use crate::{FeatureSystemDependencyArea, FeatureSystemLocation, ImportEdge, Severity, Violation};
use std::path::Path;

impl Violation {
    pub(crate) fn feature_system_layout(
        file: &Path,
        severity: Severity,
        message: String,
        suggestion: String,
    ) -> Self {
        let mut violation = base_violation(RULE_FEATURE_SYSTEM_LAYOUT, severity, file, message);
        violation.suggestion = Some(suggestion);
        violation
    }

    pub(crate) fn feature_system_public_api_wildcard(file: &Path, severity: Severity) -> Self {
        let mut violation = base_violation(
            RULE_FEATURE_SYSTEM_PUBLIC_API,
            severity,
            file,
            "feature system public API must not use wildcard re-exports",
        );
        violation.suggestion = Some(
            "replace wildcard re-exports with explicit named exports from the system barrel"
                .to_string(),
        );
        violation
    }

    pub(crate) fn feature_system_public_api_internal_import(
        edge: &ImportEdge,
        target: &Path,
        target_location: &FeatureSystemLocation,
        source_is_route: bool,
        severity: Severity,
    ) -> Self {
        let source_kind = if source_is_route {
            "route"
        } else {
            "outside code"
        };
        Self {
            rule: RULE_FEATURE_SYSTEM_PUBLIC_API.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{source_kind} may not import {} system internal file through {}",
                target_location.domain, edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: Some(target_location.domain.clone()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(format!(
                "import from the {} public entry point instead",
                target_location.system_path
            )),
            matched_layers: None,
            matched_contexts: None,
            baselined: false,
            suppressed: false,
            suppression_reason: None,
        }
    }

    pub(crate) fn feature_system_dependency_flow(
        edge: &ImportEdge,
        target: &Path,
        source_area: FeatureSystemDependencyArea,
        target_area: FeatureSystemDependencyArea,
        target_location: &FeatureSystemLocation,
        severity: Severity,
        suggestion: String,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{} may not import {} in the {} system through {}",
                source_area.display_name(),
                target_area.display_name(),
                target_location.domain,
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(source_area.config_name().to_string()),
            to_layer: Some(target_area.config_name().to_string()),
            from_context: None,
            to_context: Some(target_location.domain.clone()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
            baselined: false,
            suppressed: false,
            suppression_reason: None,
        }
    }

    pub(crate) fn feature_system_adapter_contract(
        file: &Path,
        severity: Severity,
        message: String,
        suggestion: String,
    ) -> Self {
        let mut violation = base_violation(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            severity,
            file,
            message,
        );
        violation.suggestion = Some(suggestion);
        violation
    }

    pub(crate) fn feature_system_adapter_contract_import(
        edge: &ImportEdge,
        target: &Path,
        target_area: FeatureSystemDependencyArea,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "adapter files may not import {} through {}",
                target_area.display_name(),
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some("adapters".to_string()),
            to_layer: Some(target_area.config_name().to_string()),
            from_context: None,
            to_context: None,
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "move the dependency behind lib/query-options, a store, or another upper layer"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
            baselined: false,
            suppressed: false,
            suppression_reason: None,
        }
    }

    pub(crate) fn feature_system_query_contract(
        file: &Path,
        severity: Severity,
        message: String,
        suggestion: String,
    ) -> Self {
        let mut violation =
            base_violation(RULE_FEATURE_SYSTEM_QUERY_CONTRACT, severity, file, message);
        violation.suggestion = Some(suggestion);
        violation
    }
}
