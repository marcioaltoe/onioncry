mod clean_architecture;
mod clean_boundaries;
mod code_smells;
mod feature_system;
mod repo;
mod vertical_slice;

pub(crate) use clean_architecture::collect_clean_artifact_placement_violations;
pub(crate) use clean_boundaries::{
    collect_context_cycle_violations, collect_context_violations,
    collect_external_package_violations, collect_framework_in_core_violations,
    collect_layer_violations, collect_outer_data_format_violations,
    collect_public_surface_reexport_violations, collect_unowned_schema_import_violations,
};
pub(crate) use code_smells::{
    collect_concrete_dependency_violations, collect_feature_envy_violations,
    collect_shotgun_surgery_violations,
};
pub(crate) use feature_system::{
    FeatureSystemDependencyArea, FeatureSystemLocation,
    collect_feature_system_adapter_contract_violations,
    collect_feature_system_dependency_flow_violations, collect_feature_system_layout_violations,
    collect_feature_system_public_api_violations, collect_feature_system_query_contract_violations,
};
pub(crate) use repo::{collect_path_naming_violations, collect_test_placement_violations};
pub(crate) use vertical_slice::{
    SliceLocation, collect_global_slice_artifact_violations,
    collect_vertical_shared_layer_artifact_violations,
    collect_vertical_slice_entry_point_violations,
    collect_vertical_slice_internal_import_violations,
};
