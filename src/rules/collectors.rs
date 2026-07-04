use super::{
    collect_clean_artifact_placement_violations, collect_concrete_dependency_violations,
    collect_context_cycle_violations, collect_context_violations,
    collect_external_package_violations, collect_feature_envy_violations,
    collect_feature_system_adapter_contract_violations,
    collect_feature_system_dependency_flow_violations, collect_feature_system_layout_violations,
    collect_feature_system_public_api_violations, collect_feature_system_query_contract_violations,
    collect_framework_in_core_violations, collect_global_slice_artifact_violations,
    collect_layer_violations, collect_outer_data_format_violations, collect_path_naming_violations,
    collect_public_surface_reexport_violations, collect_shotgun_surgery_violations,
    collect_test_placement_violations, collect_unowned_schema_import_violations,
    collect_vertical_shared_layer_artifact_violations,
    collect_vertical_slice_entry_point_violations,
    collect_vertical_slice_internal_import_violations,
};
use crate::{ArchitectureMode, ImportEdge, LoadedConfig, Result, RulePolicy, Violation};
use std::path::{Path, PathBuf};

pub(crate) struct RuleCollectionContext<'a> {
    pub(crate) loaded: &'a LoadedConfig,
    pub(crate) project_root: &'a Path,
    pub(crate) files: &'a [PathBuf],
    pub(crate) edges: &'a [ImportEdge],
    pub(crate) rule_policy: &'a RulePolicy,
}

type RuleCollector = fn(&RuleCollectionContext<'_>) -> Result<Vec<Violation>>;

const CLEAN_ARCHITECTURE_COLLECTORS: &[RuleCollector] = &[
    collect_layer,
    collect_external_packages,
    collect_contexts,
    collect_framework_in_core,
    collect_outer_data_format,
    collect_public_surface_reexports,
    collect_context_cycles,
    collect_unowned_schema_imports,
    collect_clean_artifact_placement,
];

const VERTICAL_SLICE_COLLECTORS: &[RuleCollector] = &[
    collect_vertical_slice_internal_imports,
    collect_global_slice_artifacts,
    collect_vertical_slice_entry_points,
    collect_vertical_shared_layer_artifacts,
];

const ARCHITECTURE_NEUTRAL_COLLECTORS: &[RuleCollector] = &[
    collect_concrete_dependencies,
    collect_feature_envy,
    collect_test_placement,
    collect_path_naming,
    collect_feature_system_layout,
    collect_feature_system_public_api,
    collect_feature_system_dependency_flow,
    collect_feature_system_adapter_contract,
    collect_feature_system_query_contract,
    collect_shotgun_surgery,
];

pub(crate) fn collect_rule_violations(
    context: &RuleCollectionContext<'_>,
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for collector in rule_collectors_for(context.loaded.config.architecture.mode) {
        violations.extend(collector(context)?);
    }
    Ok(violations)
}

fn rule_collectors_for(mode: ArchitectureMode) -> impl Iterator<Item = RuleCollector> {
    let architecture_collectors = match mode {
        ArchitectureMode::CleanArchitecture => CLEAN_ARCHITECTURE_COLLECTORS,
        ArchitectureMode::VerticalSlice => VERTICAL_SLICE_COLLECTORS,
    };

    architecture_collectors
        .iter()
        .chain(ARCHITECTURE_NEUTRAL_COLLECTORS.iter())
        .copied()
}

fn collect_layer(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_layer_violations(
        context.loaded,
        context.project_root,
        context.files,
        context.edges,
        context.rule_policy,
    )
}

fn collect_external_packages(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_external_package_violations(
        context.loaded,
        context.project_root,
        context.edges,
        context.rule_policy,
    )
}

fn collect_contexts(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_context_violations(
        context.loaded,
        context.project_root,
        context.files,
        context.edges,
        context.rule_policy,
    )
}

fn collect_framework_in_core(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_framework_in_core_violations(
        context.loaded,
        context.project_root,
        context.edges,
        context.rule_policy,
    )
}

fn collect_outer_data_format(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_outer_data_format_violations(
        context.loaded,
        context.project_root,
        context.edges,
        context.rule_policy,
    )
}

fn collect_public_surface_reexports(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_public_surface_reexport_violations(
        context.loaded,
        context.project_root,
        context.edges,
        context.rule_policy,
    )
}

fn collect_context_cycles(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_context_cycle_violations(
        context.loaded,
        context.project_root,
        context.edges,
        context.rule_policy,
    )
}

fn collect_unowned_schema_imports(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_unowned_schema_import_violations(
        context.loaded,
        context.project_root,
        context.edges,
        context.rule_policy,
    )
}

fn collect_clean_artifact_placement(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_clean_artifact_placement_violations(
        context.loaded,
        context.project_root,
        context.files,
        context.rule_policy,
    )
}

fn collect_vertical_slice_internal_imports(
    context: &RuleCollectionContext<'_>,
) -> Result<Vec<Violation>> {
    collect_vertical_slice_internal_import_violations(
        context.loaded,
        context.project_root,
        context.edges,
        context.rule_policy,
    )
}

fn collect_global_slice_artifacts(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_global_slice_artifact_violations(
        context.loaded,
        context.project_root,
        context.files,
        context.rule_policy,
    )
}

fn collect_vertical_slice_entry_points(
    context: &RuleCollectionContext<'_>,
) -> Result<Vec<Violation>> {
    collect_vertical_slice_entry_point_violations(
        context.loaded,
        context.project_root,
        context.files,
        context.rule_policy,
    )
}

fn collect_vertical_shared_layer_artifacts(
    context: &RuleCollectionContext<'_>,
) -> Result<Vec<Violation>> {
    collect_vertical_shared_layer_artifact_violations(
        context.loaded,
        context.project_root,
        context.files,
        context.rule_policy,
    )
}

fn collect_concrete_dependencies(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_concrete_dependency_violations(
        context.loaded,
        context.project_root,
        context.edges,
        context.rule_policy,
    )
}

fn collect_feature_envy(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_feature_envy_violations(
        context.loaded,
        context.project_root,
        context.edges,
        context.rule_policy,
    )
}

fn collect_test_placement(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_test_placement_violations(context.project_root, context.files, context.rule_policy)
}

fn collect_path_naming(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_path_naming_violations(context.project_root, context.files, context.rule_policy)
}

fn collect_feature_system_layout(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_feature_system_layout_violations(
        context.project_root,
        context.files,
        context.rule_policy,
    )
}

fn collect_feature_system_public_api(
    context: &RuleCollectionContext<'_>,
) -> Result<Vec<Violation>> {
    collect_feature_system_public_api_violations(
        context.project_root,
        context.files,
        context.edges,
        context.rule_policy,
    )
}

fn collect_feature_system_dependency_flow(
    context: &RuleCollectionContext<'_>,
) -> Result<Vec<Violation>> {
    collect_feature_system_dependency_flow_violations(
        context.project_root,
        context.files,
        context.edges,
        context.rule_policy,
    )
}

fn collect_feature_system_adapter_contract(
    context: &RuleCollectionContext<'_>,
) -> Result<Vec<Violation>> {
    collect_feature_system_adapter_contract_violations(
        context.project_root,
        context.files,
        context.edges,
        context.rule_policy,
    )
}

fn collect_feature_system_query_contract(
    context: &RuleCollectionContext<'_>,
) -> Result<Vec<Violation>> {
    collect_feature_system_query_contract_violations(
        context.project_root,
        context.files,
        context.edges,
        context.rule_policy,
    )
}

fn collect_shotgun_surgery(context: &RuleCollectionContext<'_>) -> Result<Vec<Violation>> {
    collect_shotgun_surgery_violations(context.project_root, context.files, context.rule_policy)
}
