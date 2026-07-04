use crate::rules::catalog::RULE_NO_FORBIDDEN_IMPORTS;
use crate::{
    ExternalPackagePolicy, ImportEdge, ImportResolution, LayerClassification, LayerClassifier,
    LoadedConfig, Result, RulePolicy, Severity, Violation, normalized_package_name,
};
use std::path::Path;

pub(crate) fn collect_external_package_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.layers.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = LayerClassifier::new(project_root, &loaded.config.layers)?;
    let mut violations = Vec::new();

    for edge in edges {
        if !matches!(edge.resolution, ImportResolution::External) {
            continue;
        }
        let LayerClassification::Classified(from_layer) = classifier.classify(&edge.source) else {
            continue;
        };
        let rule_setting =
            rule_policy.effective_rule(RULE_NO_FORBIDDEN_IMPORTS, project_root, &edge.source);
        let package_policy = ExternalPackagePolicy::from_rule_setting(&rule_setting)?;
        let layer_policy = package_policy.for_layer(from_layer);
        if layer_policy.severity == Severity::Off {
            continue;
        }

        let package_name = normalized_package_name(&edge.specifier);
        if layer_policy.allow.is_allowed(&package_name) {
            continue;
        }

        violations.push(Violation::forbidden_external_package(
            edge,
            from_layer,
            &package_name,
            layer_policy.severity,
        ));
    }

    Ok(violations)
}
