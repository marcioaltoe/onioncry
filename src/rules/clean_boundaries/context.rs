use crate::*;
use std::path::{Path, PathBuf};

pub(crate) fn collect_context_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.contexts.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = ContextClassifier::new(project_root, &loaded.config.contexts)?;
    let context_policy = ContextPolicy::from(&loaded.config.context_rules);
    let mut violations = Vec::new();

    for file in files {
        if let ContextClassification::Ambiguous(contexts) = classifier.classify(file) {
            let severity =
                rule_policy.effective_severity(RULE_AMBIGUOUS_CONTEXT, project_root, file);
            if severity != Severity::Off {
                violations.push(Violation::ambiguous_context(file, contexts, severity));
            }
        }
    }

    for edge in edges {
        let severity = rule_policy.effective_severity(
            RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT,
            project_root,
            &edge.source,
        );
        if severity == Severity::Off {
            continue;
        }
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let ContextClassification::Classified(from_context) = classifier.classify(&edge.source)
        else {
            continue;
        };
        let ContextClassification::Classified(to_context) = classifier.classify(target) else {
            continue;
        };
        if from_context == to_context && context_policy.allow_same_context {
            continue;
        }
        if from_context != to_context && context_policy.is_public_surface(target, project_root) {
            continue;
        }

        violations.push(Violation::cross_context_internal_import(
            edge,
            target,
            from_context,
            to_context,
            severity,
            &context_policy.allow_cross_context,
        ));
    }

    Ok(violations)
}

pub(crate) fn collect_unowned_schema_import_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.contexts.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = ContextClassifier::new(project_root, &loaded.config.contexts)?;
    let mut violations = Vec::new();

    for edge in edges {
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let severity = rule_policy.effective_severity(
            RULE_NO_UNOWNED_SCHEMA_IMPORT,
            project_root,
            &edge.source,
        );
        if severity == Severity::Off {
            continue;
        }
        let rule_setting =
            rule_policy.effective_rule(RULE_NO_UNOWNED_SCHEMA_IMPORT, project_root, &edge.source);
        let schema_segments = string_set_option(
            RULE_NO_UNOWNED_SCHEMA_IMPORT,
            &rule_setting,
            "schemaSegments",
            &["schema", "schemas"],
        )?;
        let schema_suffixes = string_vec_option(
            RULE_NO_UNOWNED_SCHEMA_IMPORT,
            &rule_setting,
            "schemaSuffixes",
            &[
                ".schema.ts",
                ".schema.tsx",
                ".schema.js",
                ".model.ts",
                ".model.tsx",
            ],
        )?;
        if !path_has_any_segment(target, project_root, &schema_segments)
            && !path_ends_with_any(target, project_root, &schema_suffixes)
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
        if from_context == to_context {
            continue;
        }

        violations.push(Violation::unowned_schema_import(
            edge,
            target,
            from_context,
            to_context,
            severity,
        ));
    }

    Ok(violations)
}
