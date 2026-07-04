use crate::rules::catalog::{RULE_NO_FRAMEWORK_IN_CORE, RULE_NO_OUTER_DATA_FORMAT_IN_CORE};
use crate::{
    ImportEdge, ImportResolution, LayerClassification, LayerClassifier, LoadedConfig, Result,
    RulePolicy, Severity, Violation, normalized_package_name, package_pattern_option,
    path_ends_with_any, path_has_any_segment, string_set_option, string_vec_option,
};
use std::path::Path;

pub(crate) fn collect_framework_in_core_violations(
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
        let severity =
            rule_policy.effective_severity(RULE_NO_FRAMEWORK_IN_CORE, project_root, &edge.source);
        if severity == Severity::Off {
            continue;
        };
        let rule_setting =
            rule_policy.effective_rule(RULE_NO_FRAMEWORK_IN_CORE, project_root, &edge.source);
        let core_layers = string_set_option(
            RULE_NO_FRAMEWORK_IN_CORE,
            &rule_setting,
            "coreLayers",
            &["domain", "application"],
        )?;
        let framework_packages = package_pattern_option(
            RULE_NO_FRAMEWORK_IN_CORE,
            &rule_setting,
            "packages",
            &[
                "express",
                "fastify",
                "hono",
                "koa",
                "next",
                "react",
                "vue",
                "angular",
                "@nestjs/*",
                "drizzle-orm",
                "prisma",
                "@prisma/*",
                "typeorm",
                "sequelize",
                "mongoose",
                "pg",
                "mysql2",
                "redis",
                "ioredis",
            ],
        )?;
        let LayerClassification::Classified(from_layer) = classifier.classify(&edge.source) else {
            continue;
        };
        if !core_layers.contains(from_layer) {
            continue;
        }

        let package_name = normalized_package_name(&edge.specifier);
        if !framework_packages.is_allowed(&package_name) {
            continue;
        }

        violations.push(Violation::framework_in_core(
            edge,
            from_layer,
            &package_name,
            severity,
        ));
    }

    Ok(violations)
}

pub(crate) fn collect_outer_data_format_violations(
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
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let severity = rule_policy.effective_severity(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            project_root,
            &edge.source,
        );
        if severity == Severity::Off {
            continue;
        }
        let rule_setting = rule_policy.effective_rule(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            project_root,
            &edge.source,
        );
        let core_layers = string_set_option(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            &rule_setting,
            "coreLayers",
            &["domain", "application"],
        )?;
        let outer_layers = string_set_option(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            &rule_setting,
            "outerLayers",
            &["infra"],
        )?;
        let format_segments = string_set_option(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            &rule_setting,
            "formatSegments",
            &[
                "schema", "schemas", "dto", "dtos", "record", "records", "row", "rows",
            ],
        )?;
        let format_suffixes = string_vec_option(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            &rule_setting,
            "formatSuffixes",
            &[
                ".schema.ts",
                ".schema.tsx",
                ".schema.js",
                ".dto.ts",
                ".dto.tsx",
                ".record.ts",
                ".row.ts",
                "config-types.ts",
            ],
        )?;
        let LayerClassification::Classified(from_layer) = classifier.classify(&edge.source) else {
            continue;
        };
        let LayerClassification::Classified(to_layer) = classifier.classify(target) else {
            continue;
        };
        if !core_layers.contains(from_layer) || !outer_layers.contains(to_layer) {
            continue;
        }
        if !path_has_any_segment(target, project_root, &format_segments)
            && !path_ends_with_any(target, project_root, &format_suffixes)
        {
            continue;
        }

        violations.push(Violation::outer_data_format_in_core(
            edge, target, from_layer, to_layer, severity,
        ));
    }

    Ok(violations)
}
