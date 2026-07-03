use crate::rules::catalog::RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT;
use crate::{
    ContextClassification, ContextClassifier, ContextPolicy, ImportEdge, ImportKind,
    ImportResolution, LoadedConfig, Result, RulePolicy, Severity, Violation,
};
use std::path::Path;

pub(crate) fn collect_public_surface_reexport_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.contexts.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = ContextClassifier::new(project_root, &loaded.config.contexts)?;
    let context_policy = ContextPolicy::from(&loaded.config.context_rules);
    let mut violations = Vec::new();

    for edge in edges {
        if edge.kind != ImportKind::ReExport {
            continue;
        }
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let severity = rule_policy.effective_severity(
            RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT,
            project_root,
            &edge.source,
        );
        if severity == Severity::Off {
            continue;
        }
        if !context_policy.is_public_surface(&edge.source, project_root)
            || context_policy.is_public_surface(target, project_root)
        {
            continue;
        }
        let ContextClassification::Classified(from_context) = classifier.classify(&edge.source)
        else {
            continue;
        };
        let ContextClassification::Classified(to_context) = classifier.classify(target) else {
            continue;
        };
        if from_context != to_context {
            continue;
        }

        violations.push(Violation::public_surface_internal_reexport(
            edge,
            target,
            from_context,
            severity,
        ));
    }

    Ok(violations)
}
