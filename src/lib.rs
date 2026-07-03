use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Serialize;
use std::collections::HashSet;
use std::path::Component;
use std::path::{Path, PathBuf};
use thiserror::Error;

mod classification;
mod commands;
mod config;
mod diagnostics;
mod imports;
mod policy;
mod render;
mod rules;

pub use config::{
    ArchitectureConfig, ArchitectureMode, CleanArchitectureConfig, Config, ContextConfig,
    ContextRuleDefaultConfig, ContextRulesConfig, LayerConfig, LoadedConfig, OverrideConfig,
    ProjectConfig, RuleSetting, Severity, VerticalSliceConfig,
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

pub(crate) use classification::{
    ContextClassification, ContextClassifier, ContextPolicy, LayerClassification, LayerClassifier,
};
pub use commands::{
    discover_config_path, init_config, load_config, run_check, run_explain, select_files,
};
pub use imports::collect_import_edges;
pub(crate) use policy::{
    ExternalPackageLayerPolicy, ExternalPackagePolicy, PackageAllowlist, RulePolicy,
};
pub use render::{build_report, render_explain_pretty, render_llm, render_pretty};
pub(crate) use rules::{FeatureSystemDependencyArea, FeatureSystemLocation, SliceLocation};

pub const DEFAULT_CONFIG_FILE: &str = ".onioncryrc.jsonc";
pub const JSON_CONFIG_FILE: &str = ".onioncryrc.json";
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
const RULE_UNCLASSIFIED_FILE: &str = "cleanarch/unclassified-file";
const RULE_AMBIGUOUS_LAYER: &str = "cleanarch/ambiguous-layer";
const RULE_AMBIGUOUS_CONTEXT: &str = "cleanarch/ambiguous-context";
const RULE_NO_LAYER_LEAK: &str = "cleanarch/no-layer-leak";
const RULE_NO_FORBIDDEN_IMPORTS: &str = "cleanarch/no-forbidden-imports";
const RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT: &str = "cleanarch/no-cross-context-internal-import";
const RULE_NO_FRAMEWORK_IN_CORE: &str = "cleanarch/no-framework-in-core";
const RULE_NO_OUTER_DATA_FORMAT_IN_CORE: &str = "cleanarch/no-outer-data-format-in-core";
const RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT: &str =
    "cleanarch/no-public-surface-internal-reexport";
const RULE_NO_CONTEXT_CYCLE: &str = "cleanarch/no-context-cycle";
const RULE_NO_UNOWNED_SCHEMA_IMPORT: &str = "cleanarch/no-unowned-schema-import";
const RULE_CLEAN_ARTIFACT_PLACEMENT: &str = "cleanarch/artifact-placement";
const RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT: &str =
    "verticalslice/no-cross-slice-internal-import";
const RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS: &str = "verticalslice/no-global-slice-artifacts";
const RULE_VERTICAL_SLICE_ENTRY_POINT: &str = "verticalslice/slice-entry-point";
const RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS: &str = "verticalslice/no-shared-layer-artifacts";
const RULE_NO_CONCRETE_DEPENDENCY: &str = "solid/no-concrete-dependency";
const RULE_FEATURE_ENVY: &str = "codesmells/feature-envy";
const RULE_SHOTGUN_SURGERY: &str = "codesmells/shotgun-surgery";
const RULE_TEST_PLACEMENT: &str = "repo/test-placement";
const RULE_PATH_NAMING: &str = "repo/path-naming";
const RULE_FEATURE_SYSTEM_LAYOUT: &str = "frontend/feature-system-layout";
const RULE_FEATURE_SYSTEM_PUBLIC_API: &str = "frontend/feature-system-public-api";
const RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW: &str = "frontend/feature-system-dependency-flow";
const RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT: &str = "frontend/feature-system-adapter-contract";
const RULE_FEATURE_SYSTEM_QUERY_CONTRACT: &str = "frontend/feature-system-query-contract";
const KNOWN_RULE_NAMES_DISPLAY: &str = "cleanarch/no-layer-leak, cleanarch/no-forbidden-imports, cleanarch/no-cross-context-internal-import, cleanarch/no-framework-in-core, cleanarch/no-outer-data-format-in-core, cleanarch/no-public-surface-internal-reexport, cleanarch/no-context-cycle, cleanarch/no-unowned-schema-import, cleanarch/artifact-placement, cleanarch/unclassified-file, cleanarch/ambiguous-layer, cleanarch/ambiguous-context, verticalslice/no-cross-slice-internal-import, verticalslice/no-global-slice-artifacts, verticalslice/slice-entry-point, verticalslice/no-shared-layer-artifacts, solid/no-concrete-dependency, codesmells/feature-envy, codesmells/shotgun-surgery, repo/test-placement, repo/path-naming, frontend/feature-system-layout, frontend/feature-system-public-api, frontend/feature-system-dependency-flow, frontend/feature-system-adapter-contract, frontend/feature-system-query-contract";
#[derive(Debug, Error)]
pub enum OnionCryError {
    #[error(
        "could not find {DEFAULT_CONFIG_FILE} or {JSON_CONFIG_FILE}; pass --config <path> to use a different file"
    )]
    MissingDefaultConfig,
    #[error("could not read config {path}: {source}")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("config already exists at {path}; pass --force to overwrite it")]
    ConfigAlreadyExists { path: PathBuf },
    #[error("could not write config {path}: {source}")]
    WriteConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse JSONC config {path}: {message}")]
    ParseConfig { path: PathBuf, message: String },
    #[error("project root does not exist: {path}")]
    MissingProjectRoot { path: PathBuf },
    #[error("invalid glob pattern {pattern:?}: {source}")]
    InvalidGlob {
        pattern: String,
        #[source]
        source: globset::Error,
    },
    #[error("could not read source file {path}: {source}")]
    ReadSource {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse source file {path}: {message}")]
    ParseSource { path: PathBuf, message: String },
    #[error("unknown rule {rule:?}; expected one of: {expected}")]
    UnknownRule {
        rule: String,
        expected: &'static str,
    },
    #[error("invalid value for rule {rule:?}: {message}")]
    InvalidRuleValue { rule: String, message: String },
    #[error(
        "rule {rule:?} is incompatible with architecture.mode {mode:?}; expected rules from {expected_family}"
    )]
    ArchitectureRuleModeMismatch {
        rule: String,
        mode: &'static str,
        expected_family: &'static str,
    },
}

pub type Result<T> = std::result::Result<T, OnionCryError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailOn {
    Error,
    Warning,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub status: CheckStatus,
    pub summary: CheckSummary,
    pub violations: Vec<Violation>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplainReport {
    pub file: String,
    pub layer: BoundaryExplanation,
    pub context: BoundaryExplanation,
    pub public_surface: bool,
    pub imports: Vec<ImportExplanation>,
    pub violations: Vec<Violation>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundaryExplanation {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub matched_patterns: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportExplanation {
    pub specifier: String,
    pub kind: String,
    pub type_only: bool,
    pub line: usize,
    pub column: usize,
    pub resolution: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_allowed: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Fail,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckSummary {
    pub file_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub violation_count: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Violation {
    pub rule: String,
    pub severity: String,
    pub message: String,
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_specifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_layer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_layer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_path: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_layers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_contexts: Option<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportEdge {
    pub source: PathBuf,
    pub specifier: String,
    pub kind: ImportKind,
    pub type_only: bool,
    pub line: usize,
    pub column: usize,
    pub resolution: ImportResolution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportKind {
    StaticImport,
    ReExport,
    DynamicImport,
    Require,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportResolution {
    Local(PathBuf),
    External,
    UnresolvedLocal,
}

impl CheckReport {
    pub fn should_exit_with_failure(&self) -> bool {
        matches!(self.status, CheckStatus::Fail)
    }
}

impl CheckStatus {
    fn as_str(&self) -> &'static str {
        match self {
            CheckStatus::Pass => "pass",
            CheckStatus::Fail => "fail",
        }
    }
}

impl ImportKind {
    fn as_str(self) -> &'static str {
        match self {
            ImportKind::StaticImport => "staticImport",
            ImportKind::ReExport => "reExport",
            ImportKind::DynamicImport => "dynamicImport",
            ImportKind::Require => "require",
        }
    }
}

fn path_has_any_segment(path: &Path, project_root: &Path, segments: &HashSet<String>) -> bool {
    let relative_path = path.strip_prefix(project_root).unwrap_or(path);
    relative_path.components().any(|component| {
        let Component::Normal(segment) = component else {
            return false;
        };
        segment.to_str().is_some_and(|segment| {
            segments.contains(segment) || segments.contains(&segment.to_ascii_lowercase())
        })
    })
}

fn path_ends_with_any(path: &Path, project_root: &Path, suffixes: &[String]) -> bool {
    let relative_path = path
        .strip_prefix(project_root)
        .unwrap_or(path)
        .display()
        .to_string()
        .to_ascii_lowercase();
    suffixes
        .iter()
        .any(|suffix| relative_path.ends_with(&suffix.to_ascii_lowercase()))
}

fn normalized_package_name(specifier: &str) -> String {
    if specifier.starts_with('@') {
        let mut segments = specifier.split('/');
        let Some(scope) = segments.next() else {
            return specifier.to_string();
        };
        let Some(name) = segments.next() else {
            return specifier.to_string();
        };
        return format!("{scope}/{name}");
    }

    specifier
        .split('/')
        .next()
        .map_or_else(|| specifier.to_string(), str::to_string)
}

fn project_relative_display(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn project_relative_components(project_root: &Path, path: &Path) -> Vec<String> {
    path_components(path.strip_prefix(project_root).unwrap_or(path))
}

fn path_roots(roots: Vec<String>) -> Vec<Vec<String>> {
    roots
        .iter()
        .map(|root| path_components(Path::new(root)))
        .collect()
}

fn path_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| {
            let Component::Normal(segment) = component else {
                return None;
            };
            segment.to_str().map(str::to_string)
        })
        .collect()
}

fn path_from_components(components: &[String]) -> PathBuf {
    components.iter().collect()
}

fn path_has_prefix_components(components: &[String], root: &[String]) -> bool {
    components.len() >= root.len()
        && components
            .iter()
            .zip(root.iter())
            .all(|(component, root_component)| component == root_component)
}

fn path_under_any_root(components: &[String], roots: &[Vec<String>]) -> bool {
    roots
        .iter()
        .any(|root| path_has_prefix_components(components, root))
}

fn display_path_components(components: &[String]) -> String {
    if components.is_empty() {
        ".".to_string()
    } else {
        components.join("/")
    }
}

fn is_index_file_name(file_name: &str) -> bool {
    matches!(
        file_name,
        "index.ts" | "index.tsx" | "index.js" | "index.jsx" | "index.mts" | "index.cts"
    )
}

fn is_test_file_name(file_name: &str) -> bool {
    DEFAULT_TEST_FILE_SUFFIXES
        .iter()
        .any(|suffix| file_name.ends_with(suffix))
}

fn sorted_strings(values: &HashSet<String>) -> Vec<String> {
    let mut values = values.iter().cloned().collect::<Vec<_>>();
    values.sort();
    values
}

fn display_root(roots: &[Vec<String>]) -> String {
    roots.first().map_or_else(
        || ".".to_string(),
        |root| {
            if root.is_empty() {
                ".".to_string()
            } else {
                root.join("/")
            }
        },
    )
}

fn artifact_role_folder(role: &str) -> String {
    match role {
        "repository" => "repositories",
        "service" => "services",
        "useCase" => "use-cases",
        "entity" => "entities",
        "valueObject" => "value-objects",
        "adapter" => "adapters",
        "handler" => "handlers",
        "port" => "ports",
        other => other,
    }
    .to_string()
}

fn is_core_clean_artifact_role(role: &str) -> bool {
    matches!(role, "use-cases" | "entities" | "value-objects" | "ports")
}

fn is_kebab_case_file_name(file_name: &str) -> bool {
    source_file_stem(file_name)
        .split('.')
        .all(is_kebab_case_name)
}

fn is_kebab_case_name(name: &str) -> bool {
    if name.is_empty() || name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return false;
    }
    name.bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn source_file_stem(file_name: &str) -> &str {
    SOURCE_EXTENSIONS
        .iter()
        .find_map(|extension| file_name.strip_suffix(&format!(".{extension}")))
        .unwrap_or(file_name)
}

fn stem_matches_collection_suffix(stem: &str, suffixes: &[String]) -> bool {
    let stem = stem
        .strip_suffix(".test")
        .or_else(|| stem.strip_suffix(".spec"))
        .unwrap_or(stem);

    suffixes.iter().any(|suffix| stem.ends_with(suffix))
}

fn singular_directory_name(directory: &str) -> String {
    if let Some(stem) = directory.strip_suffix("ies") {
        return format!("{stem}y");
    }
    directory
        .strip_suffix('s')
        .map_or_else(|| directory.to_string(), str::to_string)
}

fn plural_like(segment: &str) -> bool {
    segment.ends_with('s') && segment != "shared"
}

fn is_source_file(file: &Path) -> bool {
    file.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| SOURCE_EXTENSIONS.contains(&extension))
}

fn has_wildcard_reexport(source: &str) -> bool {
    source.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("export * from")
            || line.starts_with("export * as ")
            || line.starts_with("export type * from")
            || line.starts_with("export type * as ")
    })
}

fn is_component_source_file(file: &Path) -> bool {
    file.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "tsx" | "jsx"))
}

const SOURCE_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

fn resolve_against(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

fn build_glob_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(
            Glob::new(pattern).map_err(|source| OnionCryError::InvalidGlob {
                pattern: pattern.clone(),
                source,
            })?,
        );
    }
    builder
        .build()
        .map_err(|source| OnionCryError::InvalidGlob {
            pattern: patterns.join(", "),
            source,
        })
}
