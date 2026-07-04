mod defaults;
mod rule_catalog;
mod rule_options;
mod template;
mod types;

pub(crate) use defaults::{
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
    DEFAULT_TEST_FILE_SUFFIXES,
};
pub(crate) use rule_catalog::{
    default_rule_setting, parse_external_package_layer_policies, parse_rule_map,
    validate_architecture_rule_mode,
};
pub(crate) use rule_options::{
    bool_option, package_pattern_option, string_option, string_set_map_option, string_set_option,
    string_vec_option, suffix_map_option, usize_option,
};
pub use template::CONFIG_SCHEMA_URL;
pub(crate) use template::INIT_CONFIG_TEMPLATE;
pub use types::{
    ArchitectureConfig, ArchitectureMode, CleanArchitectureConfig, Config, ContextConfig,
    ContextRuleDefaultConfig, ContextRulesConfig, LayerConfig, LoadedConfig, OverrideConfig,
    ProjectConfig, RuleSetting, Severity, VerticalSliceConfig,
};
