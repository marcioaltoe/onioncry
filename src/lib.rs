mod baseline;
mod classification;
mod commands;
mod config;
mod diagnostics;
mod errors;
mod imports;
mod model;
mod path_utils;
mod policy;
mod render;
mod rules;
mod suppressions;

pub use config::{
    ArchitectureConfig, ArchitectureMode, CONFIG_SCHEMA_URL, CleanArchitectureConfig, Config,
    ContextConfig, ContextRuleDefaultConfig, ContextRulesConfig, LayerConfig, LoadedConfig,
    OverrideConfig, ProjectConfig, RuleSetting, Severity, VerticalSliceConfig,
};
pub(crate) use config::{
    DEFAULT_ALLOWED_SHARED_COMPONENT_ROOTS, DEFAULT_COLLECTION_DIRECTORIES,
    DEFAULT_COMPONENT_DIRECTORIES, DEFAULT_FEATURE_ROOTS,
    DEFAULT_FEATURE_SYSTEM_ADAPTER_BRIDGE_FILES, DEFAULT_FEATURE_SYSTEM_ADAPTER_DIRECTORY,
    DEFAULT_FEATURE_SYSTEM_ADAPTER_FILE_TEMPLATE, DEFAULT_FEATURE_SYSTEM_ALLOWED_IMPORTS,
    DEFAULT_FEATURE_SYSTEM_API_ERROR_TEMPLATE, DEFAULT_FEATURE_SYSTEM_API_EXPORT_TEMPLATE,
    DEFAULT_FEATURE_SYSTEM_HTTP_CLIENT_NAMES, DEFAULT_FEATURE_SYSTEM_OPTIONAL_DIRECTORIES,
    DEFAULT_FEATURE_SYSTEM_PUBLIC_ENTRY_POINTS, DEFAULT_FEATURE_SYSTEM_QUERY_KEYS_FILE,
    DEFAULT_FEATURE_SYSTEM_QUERY_OPTIONS_FILE, DEFAULT_FEATURE_SYSTEM_REQUIRED_DIRECTORIES,
    DEFAULT_FEATURE_SYSTEM_ROOT_INDEX_FILE, DEFAULT_IGNORED_PATH_DIRECTORIES,
    DEFAULT_LAYER_DIRECTORIES, DEFAULT_LEGACY_FEATURE_ROOTS, DEFAULT_ROUTE_ROOTS,
    DEFAULT_SUFFIXES_BY_COLLECTION, DEFAULT_SURFACE_CSS_NAME_TEMPLATE, DEFAULT_SYSTEMS_ROOTS,
    DEFAULT_TEST_FILE_SUFFIXES, INIT_CONFIG_TEMPLATE, bool_option, default_rule_setting,
    package_pattern_option, parse_external_package_layer_policies, parse_rule_map, string_option,
    string_set_map_option, string_set_option, string_vec_option, suffix_map_option, usize_option,
    validate_architecture_rule_mode,
};

pub use baseline::{BaselineEntry, ViolationBaseline};
pub(crate) use classification::{
    ContextClassification, ContextClassifier, ContextPolicy, LayerClassification, LayerClassifier,
};
pub use commands::{
    BaselineWarning, BaselineWrite, CheckOptions, CheckOutcome, discover_config_path, init_config,
    load_config, render_config_schema_json, run_check, run_check_with_options, run_explain,
    select_files, write_config_schema,
};
pub use errors::{OnionCryError, Result};
pub use imports::collect_import_edges;
pub use model::{
    BoundaryExplanation, CheckReport, CheckStatus, CheckSummary, ExplainReport, FailOn, ImportEdge,
    ImportExplanation, ImportKind, ImportResolution, Violation,
};
pub(crate) use path_utils::*;
pub(crate) use policy::{
    ExternalPackageLayerPolicy, ExternalPackagePolicy, PackageAllowlist, RulePolicy,
};
pub use render::render_rules_pretty;
pub use render::{build_report, render_explain_pretty, render_llm, render_pretty, render_sarif};
pub(crate) use rules::catalog::*;
pub use rules::catalog::{RuleCatalogEntry, rule_catalog};
pub(crate) use rules::{FeatureSystemDependencyArea, FeatureSystemLocation, SliceLocation};
pub(crate) use suppressions::apply_inline_suppressions;

pub const DEFAULT_CONFIG_FILE: &str = ".onioncryrc.jsonc";
pub const JSON_CONFIG_FILE: &str = ".onioncryrc.json";
pub const DEFAULT_BASELINE_FILE: &str = baseline::DEFAULT_BASELINE_FILE;
pub const BASELINE_VERSION: u8 = baseline::BASELINE_VERSION;
pub const BUILD_REVISION: &str = env!("ONIONCRY_BUILD_REVISION");
pub const CLI_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " revision: ",
    env!("ONIONCRY_BUILD_REVISION")
);
pub const LLM_REPORT_SEPARATOR: &str =
    "---------------------------------------------------------------------------------------";
pub const LLM_REPORT_METADATA: &str = concat!(
    "onioncry-llm-report v1 revision: ",
    env!("ONIONCRY_BUILD_REVISION")
);
