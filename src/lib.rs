use globset::{Glob, GlobSet, GlobSetBuilder};
use jsonc_parser::{ParseOptions, parse_to_serde_value};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression, ImportExpression};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;
use walkdir::WalkDir;

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
const DEFAULT_TEST_FILE_SUFFIXES: &[&str] = &[
    ".test.ts",
    ".test.tsx",
    ".test.js",
    ".test.jsx",
    ".test.mts",
    ".test.cts",
    ".test.mjs",
    ".test.cjs",
    ".spec.ts",
    ".spec.tsx",
    ".spec.js",
    ".spec.jsx",
    ".spec.mts",
    ".spec.cts",
    ".spec.mjs",
    ".spec.cjs",
];
const DEFAULT_COLLECTION_DIRECTORIES: &[&str] = &[
    "entities",
    "repositories",
    "value-objects",
    "use-cases",
    "events",
    "services",
    "gateways",
    "dtos",
];
const DEFAULT_LAYER_DIRECTORIES: &[&str] = &["domain", "application", "infra", "shared"];
const DEFAULT_FEATURE_ROOTS: &[&str] = &["src"];
const DEFAULT_IGNORED_PATH_DIRECTORIES: &[&str] = &["__tests__"];
const DEFAULT_SUFFIXES_BY_COLLECTION: &[(&str, &[&str])] = &[
    ("repositories", &[".repository"]),
    ("value-objects", &[".value-object"]),
    ("use-cases", &[".use-case"]),
    ("events", &[".event"]),
    ("services", &[".service"]),
    ("gateways", &[".gateway"]),
    ("dtos", &[".dto"]),
];
const DEFAULT_CLEAN_CONTEXT_ROOT: &str = "contexts";
const DEFAULT_CLEAN_LAYER_ALIASES: &[(&str, &[&str])] = &[
    ("domain", &["domain"]),
    ("application", &["application"]),
    ("infra", &["infra", "infrastructure"]),
];
const DEFAULT_CLEAN_ARTIFACT_FOLDERS: &[(&str, &[&str])] = &[
    (
        "domain",
        &[
            "entities",
            "value-objects",
            "aggregates",
            "events",
            "services",
            "errors",
        ],
    ),
    (
        "application",
        &[
            "use-cases",
            "ports",
            "dtos",
            "mappers",
            "services",
            "events",
        ],
    ),
    (
        "infra",
        &[
            "repositories",
            "adapters",
            "controllers",
            "database",
            "workflows",
            "bootstrap",
        ],
    ),
];
const DEFAULT_CLEAN_ARTIFACT_SUFFIXES: &[(&str, &[&str])] = &[
    (
        "repository",
        &[
            ".repository.ts",
            "-repository.ts",
            "-catalog.ts",
            ".writer.ts",
            "-writer.ts",
            "-writers.ts",
        ],
    ),
    ("service", &[".service.ts", "-service.ts"]),
    ("useCase", &[".use-case.ts", "-use-case.ts"]),
    ("entity", &[".entity.ts", "-entity.ts"]),
    ("valueObject", &[".value-object.ts", "-value-object.ts"]),
    (
        "adapter",
        &[
            ".adapter.ts",
            "-adapter.ts",
            ".gateway.ts",
            "-gateway.ts",
            "/client.ts",
            ".client.ts",
            "-client.ts",
            "/handler.ts",
            ".mapper.ts",
            "-mapper.ts",
            "-mappers.ts",
            ".parser.ts",
            "-parser.ts",
            ".provider.ts",
            "-provider.ts",
            ".request.ts",
            "-request.ts",
            "-requests.ts",
            ".schema.ts",
            "-schema.ts",
            "-schemas.ts",
            "-normalization.ts",
            "-resilience.ts",
            "-composition.ts",
            "-scenario.ts",
            "-scenarios.ts",
            "-snapshot.ts",
            "-snapshots.ts",
        ],
    ),
    ("handler", &[".handler.ts", "-handler.ts"]),
    ("port", &[".port.ts", "-port.ts", "-ports.ts"]),
];
const DEFAULT_CLEAN_GROUPED_ARTIFACT_FOLDERS: &[&str] = &[
    "use-cases",
    "entities",
    "value-objects",
    "ports",
    "repositories",
    "adapters",
    "controllers",
    "database",
    "workflows",
    "bootstrap",
];
const DEFAULT_VERTICAL_SLICE_ROOT: &str = "features";
const DEFAULT_VERTICAL_SLICE_DEPTH: usize = 2;
const DEFAULT_VERTICAL_PUBLIC_SURFACE: &[&str] = &["index.ts", "contracts"];
const DEFAULT_VERTICAL_ARTIFACT_FOLDERS: &[&str] = &["handlers", "adapters", "domain", "__tests__"];
const DEFAULT_VERTICAL_ARTIFACT_SUFFIXES: &[(&str, &[&str])] = &[
    ("repository", &[".repository.ts"]),
    ("service", &[".service.ts"]),
    ("handler", &[".handler.ts"]),
    ("adapter", &[".adapter.ts"]),
    ("entity", &[".entity.ts"]),
    ("valueObject", &[".value-object.ts"]),
    ("useCase", &[".use-case.ts"]),
];
const DEFAULT_VERTICAL_ALLOWED_GLOBAL_FOLDERS: &[&str] =
    &["app", "config", "lib", "shared", "platform"];
const DEFAULT_VERTICAL_ENTRY_POINT_NAMES: &[&str] = &[
    "setup",
    "Setup",
    "map",
    "Map",
    "register",
    "Register",
    "registerRoute",
    "RegisterRoute",
];
const DEFAULT_VERTICAL_SHARED_LAYER_FOLDERS: &[&str] = &[
    "controllers",
    "handlers",
    "services",
    "repositories",
    "use-cases",
];
const DEFAULT_SYSTEMS_ROOTS: &[&str] = &["packages/frontend/src/systems"];
const DEFAULT_FEATURE_SYSTEM_REQUIRED_DIRECTORIES: &[&str] = &["components", "lib"];
const DEFAULT_FEATURE_SYSTEM_OPTIONAL_DIRECTORIES: &[&str] =
    &["hooks", "adapters", "contexts", "stores", "guards"];
const DEFAULT_ALLOWED_SHARED_COMPONENT_ROOTS: &[&str] = &["packages/frontend/src/components/ui"];
const DEFAULT_LEGACY_FEATURE_ROOTS: &[&str] = &["packages/frontend/src/features"];
const DEFAULT_COMPONENT_DIRECTORIES: &[&str] = &["components"];
const DEFAULT_FEATURE_SYSTEM_ROOT_INDEX_FILE: &str = "index.ts";
const DEFAULT_SURFACE_CSS_NAME_TEMPLATE: &str = "{domain}.css";
const DEFAULT_ROUTE_ROOTS: &[&str] = &["packages/frontend/src/routes", "packages/frontend/src/app"];
const DEFAULT_FEATURE_SYSTEM_PUBLIC_ENTRY_POINTS: &[&str] = &["index.ts"];
const DEFAULT_FEATURE_SYSTEM_ADAPTER_BRIDGE_FILES: &[&str] = &["lib/query-options.ts"];
const DEFAULT_FEATURE_SYSTEM_ADAPTER_DIRECTORY: &str = "adapters";
const DEFAULT_FEATURE_SYSTEM_ADAPTER_FILE_TEMPLATE: &str = "{domain}-api.ts";
const DEFAULT_FEATURE_SYSTEM_API_EXPORT_TEMPLATE: &str = "{domainCamel}Api";
const DEFAULT_FEATURE_SYSTEM_API_ERROR_TEMPLATE: &str = "{DomainPascal}ApiError";
const DEFAULT_FEATURE_SYSTEM_HTTP_CLIENT_NAMES: &[&str] =
    &["fetch", "http.get", "apiClient.get", "client.get"];
const DEFAULT_FEATURE_SYSTEM_QUERY_KEYS_FILE: &str = "lib/query-keys.ts";
const DEFAULT_FEATURE_SYSTEM_QUERY_OPTIONS_FILE: &str = "lib/query-options.ts";
const DEFAULT_FEATURE_SYSTEM_ALLOWED_IMPORTS: &[(&str, &[&str])] = &[
    ("adapters", &["adapters", "lib"]),
    ("lib", &["lib"]),
    ("hooks", &["hooks", "lib"]),
    ("contexts", &["contexts", "hooks", "lib"]),
    ("stores", &["stores", "lib", "adapters"]),
    (
        "components",
        &["components", "hooks", "contexts", "stores", "lib"],
    ),
    ("guards", &["guards", "hooks", "lib"]),
    (
        "public-entry",
        &[
            "public-entry",
            "adapters",
            "lib",
            "hooks",
            "contexts",
            "stores",
            "components",
            "guards",
        ],
    ),
];
const INIT_CONFIG_TEMPLATE: &str = r#"{
  "$schema": "./onioncry.schema.json",
  "version": 1,
  "project": {
    "root": ".",
    // TODO: adjust the file universe for your source layout.
    "include": ["src/**/*.{ts,tsx,js,jsx,mts,cts,mjs,cjs}"],
    "exclude": ["node_modules/**", "dist/**", "build/**", "coverage/**"]
  },
  "architecture": {
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "contexts",
      "layerPathAliases": {
        "domain": ["domain"],
        "application": ["application"],
        "infra": ["infra", "infrastructure"]
      },
      "artifactFolders": {
        "domain": ["entities", "value-objects", "aggregates", "events", "services", "errors"],
        "application": ["use-cases", "ports", "dtos", "mappers", "services", "events"],
        "infra": ["repositories", "adapters", "controllers", "database", "workflows", "bootstrap"]
      },
      "artifactSuffixes": {
        "repository": [".repository.ts", "-repository.ts", "-catalog.ts", ".writer.ts", "-writer.ts", "-writers.ts"],
        "service": [".service.ts", "-service.ts"],
        "useCase": [".use-case.ts", "-use-case.ts"],
        "entity": [".entity.ts", "-entity.ts"],
        "valueObject": [".value-object.ts", "-value-object.ts"],
        "adapter": [
          ".adapter.ts",
          "-adapter.ts",
          ".gateway.ts",
          "-gateway.ts",
          "/client.ts",
          ".client.ts",
          "-client.ts",
          "/handler.ts",
          ".mapper.ts",
          "-mapper.ts",
          "-mappers.ts",
          ".parser.ts",
          "-parser.ts",
          ".provider.ts",
          "-provider.ts",
          ".request.ts",
          "-request.ts",
          "-requests.ts",
          ".schema.ts",
          "-schema.ts",
          "-schemas.ts",
          "-normalization.ts",
          "-resilience.ts",
          "-composition.ts",
          "-scenario.ts",
          "-scenarios.ts",
          "-snapshot.ts",
          "-snapshots.ts"
        ],
        "handler": [".handler.ts", "-handler.ts"],
        "port": [".port.ts", "-port.ts", "-ports.ts"]
      },
      "groupedArtifactFolders": [
        "use-cases",
        "entities",
        "value-objects",
        "ports",
        "repositories",
        "adapters",
        "controllers",
        "database",
        "workflows",
        "bootstrap"
      ]
    },
    "verticalSlice": {
      "sliceRoot": "features",
      "sliceDepth": 2,
      "publicSurface": ["index.ts", "contracts"],
      "artifactFolders": ["handlers", "adapters", "domain", "__tests__"],
      "artifactSuffixes": {
        "repository": [".repository.ts"],
        "service": [".service.ts"],
        "handler": [".handler.ts"],
        "adapter": [".adapter.ts"],
        "entity": [".entity.ts"],
        "valueObject": [".value-object.ts"],
        "useCase": [".use-case.ts"]
      },
      "allowedGlobalFolders": ["app", "config", "lib", "shared", "platform"],
      "entryPointNames": ["setup", "Setup", "map", "Map", "register", "Register"],
      "sharedLayerFolders": ["controllers", "handlers", "services", "repositories", "use-cases"]
    }
  },
  // TODO: map import aliases used by your project.
  "aliases": {
    "@app/": "src/"
  },
  "layers": {
    "domain": {
      // TODO: point this at your core business model.
      "patterns": ["src/domain/**"],
      "mayImport": ["domain", "shared"]
    },
    "application": {
      // TODO: point this at your use cases and application services.
      "patterns": ["src/application/**"],
      "mayImport": ["application", "domain", "shared"]
    },
    "infra": {
      // TODO: point this at adapters, frameworks, drivers, and runtime details.
      "patterns": ["src/infra/**"],
      "mayImport": ["infra", "application", "domain", "shared"]
    },
    "shared": {
      // TODO: keep shared small and stable.
      "patterns": ["src/shared/**"],
      "mayImport": ["shared"]
    }
  },
  "contexts": {
    // TODO: replace these with your bounded contexts.
    "sales": {
      "patterns": ["src/sales/**"]
    },
    "billing": {
      "patterns": ["src/billing/**"]
    }
  },
  "contextRules": {
    "default": {
      "allowSameContext": true,
      "allowCrossContext": ["contracts", "events", "ports", "shared"]
    }
  },
  "rules": {
    "cleanarch/no-layer-leak": "error",
    "cleanarch/no-cross-context-internal-import": "error",
    "cleanarch/no-forbidden-imports": ["error", {
      "layers": [
        {
          "fromLayer": "domain",
          "severity": "error",
          // TODO: allow only domain-safe packages.
          "allow": ["uuid"]
        },
        {
          "fromLayer": "application",
          "severity": "warn",
          // TODO: allow orchestration packages when they are intentional.
          "allow": []
        },
        {
          "fromLayer": "infra",
          "severity": "off",
          // TODO: infra is open by default; tighten this when useful.
          "allow": []
        }
      ]
    }],
    "cleanarch/no-framework-in-core": "warn",
    "cleanarch/no-outer-data-format-in-core": "warn",
    "cleanarch/no-public-surface-internal-reexport": "warn",
    "cleanarch/no-context-cycle": "warn",
    "cleanarch/no-unowned-schema-import": "warn",
    "cleanarch/artifact-placement": "warn",
    "solid/no-concrete-dependency": "warn",
    "codesmells/feature-envy": "warn",
    "codesmells/shotgun-surgery": "off",
    "cleanarch/unclassified-file": "warn"
    // If you switch architecture.mode to "verticalSlice", remove cleanarch/* rules and enable:
    // "verticalslice/no-cross-slice-internal-import": "warn",
    // "verticalslice/no-global-slice-artifacts": "warn",
    // "verticalslice/slice-entry-point": "warn",
    // "verticalslice/no-shared-layer-artifacts": "warn"
  },
  // TODO: use overrides for temporary policy exceptions, not file selection.
  "overrides": []
}
"#;

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub version: Value,
    pub project: ProjectConfig,
    #[serde(default)]
    pub architecture: ArchitectureConfig,
    #[serde(default)]
    pub aliases: Map<String, Value>,
    #[serde(default)]
    pub layers: BTreeMap<String, LayerConfig>,
    #[serde(default)]
    pub contexts: BTreeMap<String, ContextConfig>,
    #[serde(default)]
    pub context_rules: ContextRulesConfig,
    #[serde(default)]
    pub rules: Map<String, Value>,
    #[serde(default)]
    pub overrides: Vec<OverrideConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchitectureConfig {
    #[serde(default)]
    pub mode: ArchitectureMode,
    #[serde(default)]
    pub clean_architecture: CleanArchitectureConfig,
    #[serde(default)]
    pub vertical_slice: VerticalSliceConfig,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ArchitectureMode {
    #[default]
    CleanArchitecture,
    VerticalSlice,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanArchitectureConfig {
    #[serde(default = "default_clean_context_root")]
    pub context_root: String,
    #[serde(default = "default_clean_layer_path_aliases")]
    pub layer_path_aliases: BTreeMap<String, Vec<String>>,
    #[serde(default = "default_clean_artifact_folders")]
    pub artifact_folders: BTreeMap<String, Vec<String>>,
    #[serde(default = "default_clean_artifact_suffixes")]
    pub artifact_suffixes: BTreeMap<String, Vec<String>>,
    #[serde(default = "default_clean_grouped_artifact_folders")]
    pub grouped_artifact_folders: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerticalSliceConfig {
    #[serde(default = "default_vertical_slice_root")]
    pub slice_root: String,
    #[serde(default = "default_vertical_slice_depth")]
    pub slice_depth: usize,
    #[serde(default = "default_vertical_public_surface")]
    pub public_surface: Vec<String>,
    #[serde(default = "default_vertical_artifact_folders")]
    pub artifact_folders: Vec<String>,
    #[serde(default = "default_vertical_artifact_suffixes")]
    pub artifact_suffixes: BTreeMap<String, Vec<String>>,
    #[serde(default = "default_vertical_allowed_global_folders")]
    pub allowed_global_folders: Vec<String>,
    #[serde(default = "default_vertical_entry_point_names")]
    pub entry_point_names: Vec<String>,
    #[serde(default = "default_vertical_shared_layer_folders")]
    pub shared_layer_folders: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    #[serde(default = "default_project_root")]
    pub root: String,
    #[serde(default = "default_include_patterns")]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub may_import: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextRulesConfig {
    #[serde(default)]
    pub default: ContextRuleDefaultConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextRuleDefaultConfig {
    #[serde(default = "default_allow_same_context")]
    pub allow_same_context: bool,
    #[serde(default)]
    pub allow_cross_context: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverrideConfig {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub rules: Map<String, Value>,
}

#[derive(Debug)]
pub struct LoadedConfig {
    pub path: PathBuf,
    pub config_dir: PathBuf,
    pub config: Config,
}

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

pub fn discover_config_path(cwd: &Path, explicit_path: Option<&Path>) -> Result<PathBuf> {
    match explicit_path {
        Some(path) => Ok(resolve_against(cwd, path)),
        None => {
            for file_name in [DEFAULT_CONFIG_FILE, JSON_CONFIG_FILE] {
                let path = cwd.join(file_name);
                if path.exists() {
                    return Ok(path);
                }
            }
            Err(OnionCryError::MissingDefaultConfig)
        }
    }
}

pub fn load_config(cwd: &Path, explicit_path: Option<&Path>) -> Result<LoadedConfig> {
    let path = discover_config_path(cwd, explicit_path)?;
    let contents = fs::read_to_string(&path).map_err(|source| OnionCryError::ReadConfig {
        path: path.clone(),
        source,
    })?;
    let config =
        parse_to_serde_value::<Config>(&contents, &ParseOptions::default()).map_err(|source| {
            OnionCryError::ParseConfig {
                path: path.clone(),
                message: source.to_string(),
            }
        })?;
    let config_dir = path
        .parent()
        .map_or_else(|| cwd.to_path_buf(), Path::to_path_buf);

    Ok(LoadedConfig {
        path,
        config_dir,
        config,
    })
}

pub fn init_config(cwd: &Path, force: bool) -> Result<PathBuf> {
    let path = cwd.join(DEFAULT_CONFIG_FILE);
    if path.exists() && !force {
        return Err(OnionCryError::ConfigAlreadyExists { path });
    }

    fs::write(&path, INIT_CONFIG_TEMPLATE).map_err(|source| OnionCryError::WriteConfig {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

pub fn run_check(
    cwd: &Path,
    explicit_config: Option<&Path>,
    fail_on: FailOn,
) -> Result<CheckReport> {
    let loaded = load_config(cwd, explicit_config)?;
    let rule_policy = RulePolicy::new(&loaded.config)?;
    let files = select_files(&loaded)?;
    let project_root = loaded.project_root()?;
    let edges = collect_import_edges(&loaded, &project_root, &files)?;
    let violations = collect_all_violations(&loaded, &project_root, &files, &edges, &rule_policy)?;
    Ok(build_report(files.len(), &violations, fail_on))
}

pub fn run_explain(
    cwd: &Path,
    explicit_config: Option<&Path>,
    file: &Path,
) -> Result<ExplainReport> {
    let loaded = load_config(cwd, explicit_config)?;
    let rule_policy = RulePolicy::new(&loaded.config)?;
    let files = select_files(&loaded)?;
    let project_root = loaded.project_root()?;
    let edges = collect_import_edges(&loaded, &project_root, &files)?;
    let target_file = normalize_path(&resolve_against(cwd, file));
    let violations = collect_all_violations(&loaded, &project_root, &files, &edges, &rule_policy)?
        .into_iter()
        .filter(|violation| violation.file == target_file.display().to_string())
        .collect::<Vec<_>>();
    let imports = edges
        .iter()
        .filter(|edge| edge.source == target_file)
        .map(|edge| explain_import(&loaded, &project_root, edge, &rule_policy))
        .collect::<Result<Vec<_>>>()?;
    let context_policy = ContextPolicy::from(&loaded.config.context_rules);

    Ok(ExplainReport {
        file: target_file.display().to_string(),
        layer: explain_layer_boundary(&loaded.config.layers, &project_root, &target_file)?,
        context: explain_context_boundary(&loaded.config.contexts, &project_root, &target_file)?,
        public_surface: context_policy.is_public_surface(&target_file, &project_root),
        imports,
        violations,
    })
}

fn collect_all_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    match loaded.config.architecture.mode {
        ArchitectureMode::CleanArchitecture => {
            violations.extend(collect_layer_violations(
                loaded,
                project_root,
                files,
                edges,
                rule_policy,
            )?);
            violations.extend(collect_external_package_violations(
                loaded,
                project_root,
                edges,
                rule_policy,
            )?);
            violations.extend(collect_context_violations(
                loaded,
                project_root,
                files,
                edges,
                rule_policy,
            )?);
            violations.extend(collect_framework_in_core_violations(
                loaded,
                project_root,
                edges,
                rule_policy,
            )?);
            violations.extend(collect_outer_data_format_violations(
                loaded,
                project_root,
                edges,
                rule_policy,
            )?);
            violations.extend(collect_public_surface_reexport_violations(
                loaded,
                project_root,
                edges,
                rule_policy,
            )?);
            violations.extend(collect_context_cycle_violations(
                loaded,
                project_root,
                edges,
                rule_policy,
            )?);
            violations.extend(collect_unowned_schema_import_violations(
                loaded,
                project_root,
                edges,
                rule_policy,
            )?);
            violations.extend(collect_clean_artifact_placement_violations(
                loaded,
                project_root,
                files,
                rule_policy,
            )?);
        }
        ArchitectureMode::VerticalSlice => {
            violations.extend(collect_vertical_slice_internal_import_violations(
                loaded,
                project_root,
                edges,
                rule_policy,
            )?);
            violations.extend(collect_global_slice_artifact_violations(
                loaded,
                project_root,
                files,
                rule_policy,
            )?);
            violations.extend(collect_vertical_slice_entry_point_violations(
                loaded,
                project_root,
                files,
                rule_policy,
            )?);
            violations.extend(collect_vertical_shared_layer_artifact_violations(
                loaded,
                project_root,
                files,
                rule_policy,
            )?);
        }
    }
    violations.extend(collect_concrete_dependency_violations(
        loaded,
        project_root,
        edges,
        rule_policy,
    )?);
    violations.extend(collect_feature_envy_violations(
        loaded,
        project_root,
        edges,
        rule_policy,
    )?);
    violations.extend(collect_test_placement_violations(
        project_root,
        files,
        rule_policy,
    )?);
    violations.extend(collect_path_naming_violations(
        project_root,
        files,
        rule_policy,
    )?);
    violations.extend(collect_feature_system_layout_violations(
        project_root,
        files,
        rule_policy,
    )?);
    violations.extend(collect_feature_system_public_api_violations(
        project_root,
        files,
        edges,
        rule_policy,
    )?);
    violations.extend(collect_feature_system_dependency_flow_violations(
        project_root,
        files,
        edges,
        rule_policy,
    )?);
    violations.extend(collect_feature_system_adapter_contract_violations(
        project_root,
        files,
        edges,
        rule_policy,
    )?);
    violations.extend(collect_feature_system_query_contract_violations(
        project_root,
        files,
        edges,
        rule_policy,
    )?);
    violations.extend(collect_shotgun_surgery_violations(
        project_root,
        files,
        rule_policy,
    )?);
    Ok(violations)
}

fn explain_import(
    loaded: &LoadedConfig,
    project_root: &Path,
    edge: &ImportEdge,
    rule_policy: &RulePolicy,
) -> Result<ImportExplanation> {
    let (resolution, target_file, package_name, package_allowed) = match &edge.resolution {
        ImportResolution::Local(target) => (
            "local".to_string(),
            Some(target.display().to_string()),
            None,
            None,
        ),
        ImportResolution::External => {
            let package_name = normalized_package_name(&edge.specifier);
            let package_allowed = explain_external_package_allowed(
                loaded,
                project_root,
                edge,
                rule_policy,
                &package_name,
            )?;
            (
                "external".to_string(),
                None,
                Some(package_name),
                package_allowed,
            )
        }
        ImportResolution::UnresolvedLocal => ("unresolvedLocal".to_string(), None, None, None),
    };

    Ok(ImportExplanation {
        specifier: edge.specifier.clone(),
        kind: edge.kind.as_str().to_string(),
        type_only: edge.type_only,
        line: edge.line,
        column: edge.column,
        resolution,
        target_file,
        package_name,
        package_allowed,
    })
}

fn explain_external_package_allowed(
    loaded: &LoadedConfig,
    project_root: &Path,
    edge: &ImportEdge,
    rule_policy: &RulePolicy,
    package_name: &str,
) -> Result<Option<bool>> {
    if loaded.config.layers.is_empty() {
        return Ok(None);
    }

    let classifier = LayerClassifier::new(project_root, &loaded.config.layers)?;
    let LayerClassification::Classified(from_layer) = classifier.classify(&edge.source) else {
        return Ok(None);
    };
    let rule_setting =
        rule_policy.effective_rule(RULE_NO_FORBIDDEN_IMPORTS, project_root, &edge.source);
    let package_policy = ExternalPackagePolicy::from_rule_setting(&rule_setting)?;
    let layer_policy = package_policy.for_layer(from_layer);
    if layer_policy.severity == Severity::Off {
        return Ok(Some(true));
    }

    Ok(Some(layer_policy.allow.is_allowed(package_name)))
}

fn explain_layer_boundary(
    layers: &BTreeMap<String, LayerConfig>,
    project_root: &Path,
    file: &Path,
) -> Result<BoundaryExplanation> {
    explain_boundary(
        project_root,
        file,
        layers
            .iter()
            .map(|(name, config)| (name.as_str(), config.patterns.as_slice())),
        "unclassified",
    )
}

fn explain_context_boundary(
    contexts: &BTreeMap<String, ContextConfig>,
    project_root: &Path,
    file: &Path,
) -> Result<BoundaryExplanation> {
    explain_boundary(
        project_root,
        file,
        contexts
            .iter()
            .map(|(name, config)| (name.as_str(), config.patterns.as_slice())),
        "contextless",
    )
}

fn explain_boundary<'a>(
    project_root: &Path,
    file: &Path,
    entries: impl Iterator<Item = (&'a str, &'a [String])>,
    empty_status: &str,
) -> Result<BoundaryExplanation> {
    let relative_path = file.strip_prefix(project_root).unwrap_or(file);
    let mut matched_entries = Vec::new();

    for (name, patterns) in entries {
        let mut matched_patterns = Vec::new();
        for pattern in patterns {
            let glob = Glob::new(pattern).map_err(|source| OnionCryError::InvalidGlob {
                pattern: pattern.clone(),
                source,
            })?;
            if glob.compile_matcher().is_match(relative_path) {
                matched_patterns.push(pattern.clone());
            }
        }
        if !matched_patterns.is_empty() {
            matched_entries.push((name.to_string(), matched_patterns));
        }
    }

    match matched_entries.as_slice() {
        [] => Ok(BoundaryExplanation {
            status: empty_status.to_string(),
            name: None,
            matched_patterns: Vec::new(),
        }),
        [(name, matched_patterns)] => Ok(BoundaryExplanation {
            status: "classified".to_string(),
            name: Some(name.clone()),
            matched_patterns: matched_patterns.clone(),
        }),
        entries => Ok(BoundaryExplanation {
            status: "ambiguous".to_string(),
            name: None,
            matched_patterns: entries
                .iter()
                .flat_map(|(_, patterns)| patterns.iter().cloned())
                .collect(),
        }),
    }
}

pub fn select_files(loaded: &LoadedConfig) -> Result<Vec<PathBuf>> {
    let root = normalize_path(&resolve_against(
        &loaded.config_dir,
        Path::new(&loaded.config.project.root),
    ));
    if !root.is_dir() {
        return Err(OnionCryError::MissingProjectRoot { path: root });
    }

    let include = build_glob_set(&loaded.config.project.include)?;
    let exclude = build_glob_set(&loaded.config.project.exclude)?;
    let mut files = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let relative_path = entry.path().strip_prefix(&root).unwrap_or(entry.path());
        if include.is_match(relative_path) && !exclude.is_match(relative_path) {
            files.push(entry.path().to_path_buf());
        }
    }

    files.sort();
    Ok(files)
}

pub fn collect_import_edges(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[PathBuf],
) -> Result<Vec<ImportEdge>> {
    let aliases = loaded.alias_mappings();
    let mut edges = Vec::new();

    for file in files {
        if !is_source_file(file) {
            continue;
        }
        let source = fs::read_to_string(file).map_err(|source| OnionCryError::ReadSource {
            path: file.clone(),
            source,
        })?;
        let raw_imports = scan_imports(file, &source)?;
        for raw_import in raw_imports {
            let resolution = resolve_import(&raw_import.specifier, file, project_root, &aliases);
            let (line, column) = line_column(&source, raw_import.span.start as usize);
            edges.push(ImportEdge {
                source: file.clone(),
                specifier: raw_import.specifier,
                kind: raw_import.kind,
                type_only: raw_import.type_only,
                line,
                column,
                resolution,
            });
        }
    }

    Ok(edges)
}

fn collect_layer_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.layers.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = LayerClassifier::new(project_root, &loaded.config.layers)?;
    let mut violations = Vec::new();

    for file in files {
        match classifier.classify(file) {
            LayerClassification::Classified(_) => {}
            LayerClassification::Unclassified => {
                let unclassified_severity =
                    rule_policy.effective_severity(RULE_UNCLASSIFIED_FILE, project_root, file);
                if unclassified_severity == Severity::Off {
                    continue;
                }
                violations.push(Violation::unclassified_file(file, unclassified_severity));
            }
            LayerClassification::Ambiguous(layers) => {
                let ambiguous_severity =
                    rule_policy.effective_severity(RULE_AMBIGUOUS_LAYER, project_root, file);
                if ambiguous_severity == Severity::Off {
                    continue;
                }
                violations.push(Violation::ambiguous_layer(file, layers, ambiguous_severity));
            }
        }
    }

    for edge in edges {
        let layer_leak_severity =
            rule_policy.effective_severity(RULE_NO_LAYER_LEAK, project_root, &edge.source);
        if layer_leak_severity == Severity::Off {
            continue;
        }
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let LayerClassification::Classified(from_layer) = classifier.classify(&edge.source) else {
            continue;
        };
        let LayerClassification::Classified(to_layer) = classifier.classify(target) else {
            continue;
        };
        if classifier.may_import(from_layer, to_layer) {
            continue;
        }
        violations.push(Violation::layer_leak(
            edge,
            target,
            from_layer,
            to_layer,
            layer_leak_severity,
        ));
    }

    Ok(violations)
}

fn collect_external_package_violations(
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

fn collect_context_violations(
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

fn collect_framework_in_core_violations(
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

fn collect_outer_data_format_violations(
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

fn collect_public_surface_reexport_violations(
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

fn collect_context_cycle_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.contexts.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = ContextClassifier::new(project_root, &loaded.config.contexts)?;
    let mut graph = BTreeMap::<PathBuf, BTreeSet<PathBuf>>::new();
    let mut representatives = BTreeMap::<(String, String), (&ImportEdge, PathBuf)>::new();

    for edge in edges {
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
        if from_context == to_context {
            continue;
        }

        graph
            .entry(PathBuf::from(from_context))
            .or_default()
            .insert(PathBuf::from(to_context));
        graph.entry(PathBuf::from(to_context)).or_default();
        representatives
            .entry((from_context.to_string(), to_context.to_string()))
            .or_insert((edge, target.clone()));
    }

    let graph = graph
        .into_iter()
        .map(|(source, targets)| (source, targets.into_iter().collect::<Vec<_>>()))
        .collect::<BTreeMap<_, _>>();
    let cycles = find_canonical_cycles(&graph);
    let mut violations = Vec::new();

    for cycle in cycles {
        if cycle.len() < 3 {
            continue;
        }
        let context_path = cycle
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        let Some(from_context) = context_path.first() else {
            continue;
        };
        let Some(to_context) = context_path.get(1) else {
            continue;
        };
        let Some((edge, target)) = representatives.get(&(from_context.clone(), to_context.clone()))
        else {
            continue;
        };
        let severity =
            rule_policy.effective_severity(RULE_NO_CONTEXT_CYCLE, project_root, &edge.source);
        if severity == Severity::Off {
            continue;
        }
        violations.push(Violation::context_cycle(
            edge,
            target,
            &context_path,
            severity,
        ));
    }

    Ok(violations)
}

fn collect_unowned_schema_import_violations(
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

fn collect_concrete_dependency_violations(
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
        let severity =
            rule_policy.effective_severity(RULE_NO_CONCRETE_DEPENDENCY, project_root, &edge.source);
        if severity == Severity::Off {
            continue;
        }
        let rule_setting =
            rule_policy.effective_rule(RULE_NO_CONCRETE_DEPENDENCY, project_root, &edge.source);
        let core_layers = string_set_option(
            RULE_NO_CONCRETE_DEPENDENCY,
            &rule_setting,
            "coreLayers",
            &["domain", "application"],
        )?;
        let concrete_segments = string_set_option(
            RULE_NO_CONCRETE_DEPENDENCY,
            &rule_setting,
            "concreteSegments",
            &[
                "adapter",
                "adapters",
                "controller",
                "controllers",
                "client",
                "clients",
                "provider",
                "providers",
                "driver",
                "drivers",
            ],
        )?;
        let concrete_suffixes = string_vec_option(
            RULE_NO_CONCRETE_DEPENDENCY,
            &rule_setting,
            "concreteSuffixes",
            &[
                ".adapter.ts",
                ".adapter.tsx",
                ".controller.ts",
                ".client.ts",
                ".provider.ts",
                ".repository.adapter.ts",
            ],
        )?;
        let LayerClassification::Classified(from_layer) = classifier.classify(&edge.source) else {
            continue;
        };
        if !core_layers.contains(from_layer) {
            continue;
        }

        let concrete = match &edge.resolution {
            ImportResolution::Local(target) => {
                path_has_any_segment(target, project_root, &concrete_segments)
                    || path_ends_with_any(target, project_root, &concrete_suffixes)
            }
            ImportResolution::External | ImportResolution::UnresolvedLocal => false,
        };
        if !concrete {
            continue;
        }
        let target = match &edge.resolution {
            ImportResolution::Local(target) => Some(target.as_path()),
            ImportResolution::External | ImportResolution::UnresolvedLocal => None,
        };
        violations.push(Violation::concrete_dependency(
            edge, target, from_layer, severity,
        ));
    }

    Ok(violations)
}

fn collect_feature_envy_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.contexts.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = ContextClassifier::new(project_root, &loaded.config.contexts)?;
    let mut by_source = BTreeMap::<PathBuf, FeatureImportCounts>::new();

    for edge in edges {
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
        let counts = by_source
            .entry(edge.source.clone())
            .or_insert_with(|| FeatureImportCounts::new(from_context));
        if from_context == to_context {
            counts.own_context_count += 1;
        } else {
            counts
                .other_context_counts
                .entry(to_context.to_string())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    let mut violations = Vec::new();
    for (source, counts) in by_source {
        let severity = rule_policy.effective_severity(RULE_FEATURE_ENVY, project_root, &source);
        if severity == Severity::Off {
            continue;
        }
        let rule_setting = rule_policy.effective_rule(RULE_FEATURE_ENVY, project_root, &source);
        let min_imports = usize_option(
            RULE_FEATURE_ENVY,
            &rule_setting,
            "minImportsFromOtherContext",
            3,
        )?;
        let require_more_than_own = bool_option(
            RULE_FEATURE_ENVY,
            &rule_setting,
            "requireMoreThanOwnContext",
            true,
        )?;

        for (target_context, count) in counts.other_context_counts {
            if count < min_imports {
                continue;
            }
            if require_more_than_own && count <= counts.own_context_count {
                continue;
            }
            violations.push(Violation::feature_envy(
                &source,
                &counts.source_context,
                &target_context,
                count,
                counts.own_context_count,
                severity,
            ));
        }
    }

    Ok(violations)
}

fn collect_clean_artifact_placement_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let policy =
        CleanArtifactPlacementPolicy::from_config(&loaded.config.architecture.clean_architecture);
    let file_index = CleanArtifactPlacementFileIndex::from_files(project_root, files);
    let mut violations = Vec::new();

    for file in files {
        let rule_setting =
            rule_policy.effective_rule(RULE_CLEAN_ARTIFACT_PLACEMENT, project_root, file);
        if rule_setting.severity == Severity::Off {
            continue;
        }
        if let Some(finding) = policy.finding(project_root, file, &file_index) {
            violations.push(Violation::clean_artifact_placement(
                file,
                rule_setting.severity,
                &finding.role,
                &finding.expected_layer,
                &finding.expected_boundary,
            ));
        }
    }

    Ok(violations)
}

fn collect_vertical_slice_internal_import_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let policy = VerticalSlicePolicy::from_config(&loaded.config.architecture.vertical_slice);
    let mut violations = Vec::new();

    for edge in edges {
        let severity = rule_policy.effective_severity(
            RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT,
            project_root,
            &edge.source,
        );
        if severity == Severity::Off {
            continue;
        }
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let Some(source_location) = policy.slice_location(project_root, &edge.source) else {
            continue;
        };
        let Some(target_location) = policy.slice_location(project_root, target) else {
            continue;
        };
        if source_location.slice == target_location.slice
            || policy.is_public_surface_location(&target_location)
        {
            continue;
        }

        violations.push(Violation::cross_slice_internal_import(
            edge,
            target,
            severity,
            &source_location,
            &target_location,
        ));
    }

    Ok(violations)
}

fn collect_global_slice_artifact_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let policy = VerticalSlicePolicy::from_config(&loaded.config.architecture.vertical_slice);
    let mut violations = Vec::new();

    for file in files {
        let rule_setting =
            rule_policy.effective_rule(RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS, project_root, file);
        if rule_setting.severity == Severity::Off
            || policy.slice_location(project_root, file).is_some()
            || policy.is_allowed_global_file(project_root, file)
        {
            continue;
        }
        let Some(role) = policy.slice_artifact_role(project_root, file) else {
            continue;
        };
        violations.push(Violation::global_slice_artifact(
            file,
            rule_setting.severity,
            &role,
            &policy.slice_root_pattern_display(),
        ));
    }

    Ok(violations)
}

fn collect_vertical_slice_entry_point_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let policy = VerticalSlicePolicy::from_config(&loaded.config.architecture.vertical_slice);
    let mut files_by_slice = BTreeMap::<String, (String, Vec<PathBuf>)>::new();
    let mut violations = Vec::new();

    for file in files {
        let Some(location) = policy.slice_location(project_root, file) else {
            continue;
        };
        if file
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .is_some_and(is_test_file_name)
        {
            continue;
        }
        files_by_slice
            .entry(location.slice)
            .or_insert_with(|| (location.slice_path, Vec::new()))
            .1
            .push(file.clone());
    }

    for (slice, (slice_path, slice_files)) in files_by_slice {
        let Some(representative_file) = slice_files.first() else {
            continue;
        };
        let rule_setting = rule_policy.effective_rule(
            RULE_VERTICAL_SLICE_ENTRY_POINT,
            project_root,
            representative_file,
        );
        if rule_setting.severity == Severity::Off {
            continue;
        }
        if slice_has_configured_entry_point(&slice_files, &policy.entry_point_names)? {
            continue;
        }

        violations.push(Violation::vertical_slice_entry_point(
            representative_file,
            rule_setting.severity,
            &slice,
            &slice_path,
            &policy.entry_point_names_display(),
        ));
    }

    Ok(violations)
}

fn collect_vertical_shared_layer_artifact_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let policy = VerticalSlicePolicy::from_config(&loaded.config.architecture.vertical_slice);
    let mut violations = Vec::new();

    for file in files {
        let rule_setting =
            rule_policy.effective_rule(RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS, project_root, file);
        if rule_setting.severity == Severity::Off
            || policy.slice_location(project_root, file).is_some()
        {
            continue;
        }
        let Some(folder) = policy.shared_layer_folder(project_root, file) else {
            continue;
        };
        violations.push(Violation::vertical_shared_layer_artifact(
            file,
            rule_setting.severity,
            &folder,
            &policy.slice_root_pattern_display(),
        ));
    }

    Ok(violations)
}

fn slice_has_configured_entry_point(
    files: &[PathBuf],
    entry_point_names: &[String],
) -> Result<bool> {
    for file in files {
        let source = fs::read_to_string(file).map_err(|source| OnionCryError::ReadSource {
            path: file.clone(),
            source,
        })?;
        if contains_configured_entry_point(&source, entry_point_names) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn contains_configured_entry_point(source: &str, entry_point_names: &[String]) -> bool {
    entry_point_names.iter().any(|name| {
        [
            "export function ",
            "export async function ",
            "export const ",
            "export let ",
            "export class ",
            "export { ",
            "export {",
        ]
        .iter()
        .any(|prefix| source_contains_exported_name(source, prefix, name))
    })
}

fn source_contains_exported_name(source: &str, prefix: &str, name: &str) -> bool {
    let pattern = format!("{prefix}{name}");
    source.match_indices(&pattern).any(|(index, _)| {
        source[index + pattern.len()..]
            .chars()
            .next()
            .is_none_or(|character| !is_javascript_identifier_continue(character))
    })
}

fn is_javascript_identifier_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '$'
}

fn collect_shotgun_surgery_violations(
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if files.iter().all(|file| {
        rule_policy.effective_severity(RULE_SHOTGUN_SURGERY, project_root, file) == Severity::Off
    }) {
        return Ok(Vec::new());
    }

    let history = git_change_history(project_root, files);
    shotgun_surgery_findings(project_root, files, &history, rule_policy)
}

fn collect_test_placement_violations(
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    for file in files {
        let rule_setting = rule_policy.effective_rule(RULE_TEST_PLACEMENT, project_root, file);
        if rule_setting.severity == Severity::Off {
            continue;
        }
        let policy = TestPlacementPolicy::from_rule_setting(&rule_setting)?;
        if !policy.is_test_file(project_root, file)
            || policy.is_valid_test_placement(project_root, file)
        {
            continue;
        }

        violations.push(Violation::misplaced_test_file(
            file,
            rule_setting.severity,
            policy.suggestion(project_root, file),
        ));
    }

    Ok(violations)
}

fn collect_path_naming_violations(
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    for file in files {
        let rule_setting = rule_policy.effective_rule(RULE_PATH_NAMING, project_root, file);
        if rule_setting.severity == Severity::Off {
            continue;
        }
        let policy = PathNamingPolicy::from_rule_setting(&rule_setting)?;
        violations.extend(policy.violations(project_root, file, rule_setting.severity));
    }

    Ok(violations)
}

fn collect_feature_system_layout_violations(
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting = rule_policy.effective_rule(RULE_FEATURE_SYSTEM_LAYOUT, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemLayoutPolicy::from_rule_setting(&rule_setting)?;

    Ok(policy.violations(project_root, files, rule_setting.severity))
}

fn collect_feature_system_public_api_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_PUBLIC_API, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemPublicApiPolicy::from_rule_setting(&rule_setting)?;

    policy.violations(project_root, files, edges, rule_setting.severity)
}

fn collect_feature_system_dependency_flow_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemDependencyFlowPolicy::from_rule_setting(&rule_setting)?;

    Ok(policy.violations(project_root, edges, rule_setting.severity))
}

fn collect_feature_system_adapter_contract_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemAdapterContractPolicy::from_rule_setting(&rule_setting)?;

    policy.violations(project_root, files, edges, rule_setting.severity)
}

fn collect_feature_system_query_contract_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_QUERY_CONTRACT, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemQueryContractPolicy::from_rule_setting(&rule_setting)?;

    policy.violations(project_root, files, edges, rule_setting.severity)
}

fn find_canonical_cycles(graph: &BTreeMap<PathBuf, Vec<PathBuf>>) -> Vec<Vec<PathBuf>> {
    let components = TarjanState::new(graph).strongly_connected_components();
    let mut cycles = components
        .into_iter()
        .filter_map(|component| representative_cycle(graph, component))
        .collect::<Vec<_>>();
    cycles.sort();
    cycles
}

struct TarjanState<'a> {
    graph: &'a BTreeMap<PathBuf, Vec<PathBuf>>,
    next_index: usize,
    indexes: BTreeMap<PathBuf, usize>,
    lowlinks: BTreeMap<PathBuf, usize>,
    stack: Vec<PathBuf>,
    on_stack: HashSet<PathBuf>,
    components: Vec<Vec<PathBuf>>,
}

impl<'a> TarjanState<'a> {
    fn new(graph: &'a BTreeMap<PathBuf, Vec<PathBuf>>) -> Self {
        Self {
            graph,
            next_index: 0,
            indexes: BTreeMap::new(),
            lowlinks: BTreeMap::new(),
            stack: Vec::new(),
            on_stack: HashSet::new(),
            components: Vec::new(),
        }
    }

    fn strongly_connected_components(mut self) -> Vec<Vec<PathBuf>> {
        for node in self.graph.keys() {
            if !self.indexes.contains_key(node) {
                self.visit(node.clone());
            }
        }
        self.components
    }

    fn visit(&mut self, node: PathBuf) {
        let index = self.next_index;
        self.next_index += 1;
        self.indexes.insert(node.clone(), index);
        self.lowlinks.insert(node.clone(), index);
        self.stack.push(node.clone());
        self.on_stack.insert(node.clone());

        if let Some(targets) = self.graph.get(&node) {
            for target in targets {
                if !self.graph.contains_key(target) {
                    continue;
                }
                if !self.indexes.contains_key(target) {
                    self.visit(target.clone());
                    let target_lowlink = *self
                        .lowlinks
                        .get(target)
                        .expect("visited target should have a lowlink");
                    let node_lowlink = *self
                        .lowlinks
                        .get(&node)
                        .expect("visited node should have a lowlink");
                    self.lowlinks
                        .insert(node.clone(), node_lowlink.min(target_lowlink));
                } else if self.on_stack.contains(target) {
                    let target_index = *self
                        .indexes
                        .get(target)
                        .expect("indexed target should have an index");
                    let node_lowlink = *self
                        .lowlinks
                        .get(&node)
                        .expect("visited node should have a lowlink");
                    self.lowlinks
                        .insert(node.clone(), node_lowlink.min(target_index));
                }
            }
        }

        let node_lowlink = *self
            .lowlinks
            .get(&node)
            .expect("visited node should have a lowlink");
        if node_lowlink != index {
            return;
        }

        let mut component = Vec::new();
        loop {
            let member = self
                .stack
                .pop()
                .expect("root component should have stack members");
            self.on_stack.remove(&member);
            component.push(member.clone());
            if member == node {
                break;
            }
        }
        component.sort();
        self.components.push(component);
    }
}

fn representative_cycle(
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
    component: Vec<PathBuf>,
) -> Option<Vec<PathBuf>> {
    if component.len() == 1 {
        let node = component.first()?;
        return graph.get(node).and_then(|targets| {
            targets
                .contains(node)
                .then(|| vec![node.clone(), node.clone()])
        });
    }

    let component_set = component.iter().cloned().collect::<HashSet<_>>();
    component
        .iter()
        .find_map(|start| representative_cycle_from(graph, start, &component_set))
}

fn representative_cycle_from(
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
    start: &PathBuf,
    component: &HashSet<PathBuf>,
) -> Option<Vec<PathBuf>> {
    let mut targets = graph.get(start)?.clone();
    targets.retain(|target| component.contains(target) && target != start);
    targets.sort();

    for target in targets {
        let mut cycle = vec![start.clone(), target.clone()];
        let mut visited = HashSet::from([start.clone(), target]);
        if close_cycle(graph, start, component, &mut cycle, &mut visited) {
            return Some(cycle);
        }
    }

    None
}

fn close_cycle(
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
    start: &PathBuf,
    component: &HashSet<PathBuf>,
    cycle: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) -> bool {
    let Some(current) = cycle.last().cloned() else {
        return false;
    };
    let Some(targets) = graph.get(&current) else {
        return false;
    };
    let mut targets = targets
        .iter()
        .filter(|target| component.contains(*target))
        .cloned()
        .collect::<Vec<_>>();
    targets.sort();

    for target in targets {
        if &target == start {
            cycle.push(start.clone());
            return true;
        }
        if !visited.insert(target.clone()) {
            continue;
        }
        cycle.push(target.clone());
        if close_cycle(graph, start, component, cycle, visited) {
            return true;
        }
        cycle.pop();
        visited.remove(&target);
    }

    false
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

#[derive(Clone, Debug)]
struct RawImport {
    specifier: String,
    kind: ImportKind,
    type_only: bool,
    span: Span,
}

#[derive(Clone, Debug)]
struct AliasMapping {
    prefix: String,
    target: PathBuf,
}

struct LayerClassifier {
    project_root: PathBuf,
    layers: Vec<CompiledLayer>,
}

struct CompiledLayer {
    name: String,
    patterns: GlobSet,
    may_import: HashSet<String>,
}

enum LayerClassification<'a> {
    Classified(&'a str),
    Unclassified,
    Ambiguous(Vec<String>),
}

struct ContextClassifier {
    project_root: PathBuf,
    contexts: Vec<CompiledContext>,
}

struct CompiledContext {
    name: String,
    patterns: GlobSet,
}

enum ContextClassification<'a> {
    Classified(&'a str),
    Contextless,
    Ambiguous(Vec<String>),
}

struct ContextPolicy {
    allow_same_context: bool,
    allow_cross_context: HashSet<String>,
}

struct FeatureImportCounts {
    source_context: String,
    own_context_count: usize,
    other_context_counts: BTreeMap<String, usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Off,
    Warn,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuleSetting {
    pub severity: Severity,
    pub options: Option<Value>,
}

struct RulePolicy {
    base_rules: BTreeMap<String, RuleSetting>,
    overrides: Vec<CompiledOverride>,
}

struct CompiledOverride {
    files: GlobSet,
    rules: BTreeMap<String, RuleSetting>,
}

struct ExternalPackagePolicy {
    default_severity: Severity,
    default_allow: PackageAllowlist,
    layers: Vec<ExternalPackageLayerPolicy>,
}

struct ExternalPackageLayerPolicy {
    from_layer: String,
    severity: Severity,
    allow: PackageAllowlist,
}

struct EffectiveExternalPackageLayerPolicy<'a> {
    severity: Severity,
    allow: &'a PackageAllowlist,
}

struct PackageAllowlist {
    patterns: GlobSet,
}

struct TestPlacementPolicy {
    source_roots: Vec<Vec<String>>,
    unit_test_directories: HashSet<String>,
    integration_roots: Vec<Vec<String>>,
    e2e_roots: Vec<Vec<String>>,
    test_file_suffixes: Vec<String>,
}

struct PathNamingPolicy {
    collection_directories: HashSet<String>,
    singular_collection_directories: BTreeMap<String, String>,
    feature_roots: Vec<Vec<String>>,
    layer_directories: HashSet<String>,
    ignored_directories: HashSet<String>,
    suffixes_by_collection: BTreeMap<String, Vec<String>>,
}

struct CleanArtifactPlacementPolicy {
    source_prefix: Vec<String>,
    context_root: Vec<String>,
    layer_aliases: BTreeMap<String, HashSet<String>>,
    artifact_folders_by_layer: BTreeMap<String, Vec<String>>,
    folder_layers: BTreeMap<String, BTreeSet<String>>,
    suffix_roles: BTreeMap<String, Vec<String>>,
    grouped_artifact_folders: HashSet<String>,
}

struct CleanArtifactFinding {
    role: String,
    expected_layer: String,
    expected_boundary: String,
}

struct CleanArtifactPlacementFileIndex {
    direct_source_file_counts: BTreeMap<Vec<String>, usize>,
}

struct VerticalSlicePolicy {
    slice_root: Vec<String>,
    slice_depth: usize,
    public_surface: Vec<Vec<String>>,
    artifact_folders: HashSet<String>,
    artifact_suffixes: Vec<(String, String)>,
    allowed_global_folders: HashSet<String>,
    entry_point_names: Vec<String>,
    shared_layer_folders: HashSet<String>,
}

#[derive(Clone)]
struct SliceLocation {
    slice: String,
    slice_path: String,
    relative_file: Vec<String>,
}

struct FeatureSystemLayoutPolicy {
    systems_roots: Vec<Vec<String>>,
    required_directories: HashSet<String>,
    optional_directories: HashSet<String>,
    root_index_file: String,
    allowed_shared_component_roots: Vec<Vec<String>>,
    legacy_roots: Vec<Vec<String>>,
    component_directories: HashSet<String>,
    surface_css_name_template: String,
}

struct FeatureSystem {
    domain: String,
    path: PathBuf,
    representative_file: PathBuf,
}

struct FeatureSystemPublicApiPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    allowed_public_entry_points: HashSet<String>,
    reject_wildcard_reexports: bool,
}

struct FeatureSystemDependencyFlowPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    allowed_public_entry_points: HashSet<String>,
    adapter_bridge_files: HashSet<String>,
    allowed_imports: BTreeMap<String, HashSet<String>>,
}

struct FeatureSystemAdapterContractPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    adapter_directory: String,
    adapter_file_name_template: String,
    api_export_name_template: String,
    error_export_name_template: String,
    http_client_names: Vec<String>,
}

struct FeatureSystemQueryContractPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    adapter_directory: String,
    query_keys_file: String,
    query_options_file: String,
}

struct FeatureSystemAdapterInfo {
    domain: String,
    representative_file: PathBuf,
    expected_adapter_file: PathBuf,
    expected_relative_file: String,
}

struct FeatureSystemQueryState {
    domain: String,
    system_path: String,
    representative_file: PathBuf,
    files: Vec<PathBuf>,
    requires_query_keys: bool,
    requires_query_options: bool,
}

struct FeatureSystemLocation {
    domain: String,
    system_path: String,
    relative_file: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FeatureSystemDependencyArea {
    PublicEntry,
    Adapters,
    Lib,
    Hooks,
    Contexts,
    Stores,
    Components,
    Guards,
    Routes,
    Other,
}

pub fn build_report(file_count: usize, violations: &[Violation], fail_on: FailOn) -> CheckReport {
    let warning_count = violations
        .iter()
        .filter(|violation| violation.severity == "warn")
        .count();
    let error_count = violations
        .iter()
        .filter(|violation| violation.severity == "error")
        .count();
    let should_fail = match fail_on {
        FailOn::Error => error_count > 0,
        FailOn::Warning => error_count > 0 || warning_count > 0,
    };

    CheckReport {
        status: if should_fail {
            CheckStatus::Fail
        } else {
            CheckStatus::Pass
        },
        summary: CheckSummary {
            file_count,
            warning_count,
            error_count,
            violation_count: violations.len(),
        },
        violations: violations.to_vec(),
    }
}

pub fn render_pretty(report: &CheckReport, include_tips: bool) -> String {
    let mut output = String::new();

    let mut current_file: Option<&str> = None;
    for violation in sorted_violations(&report.violations) {
        if current_file != Some(violation.file.as_str()) {
            if current_file.is_some() {
                output.push('\n');
            }
            current_file = Some(violation.file.as_str());
            output.push_str(&violation.file);
            output.push('\n');
        }
        output.push_str(&render_pretty_violation(violation, include_tips));
    }
    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str(&render_pretty_summary(report));
    output
}

pub fn render_explain_pretty(report: &ExplainReport, include_tips: bool) -> String {
    let mut output = format!(
        "file: {}\nlayer: {}\ncontext: {}\npublicSurface: {}\n",
        report.file,
        boundary_summary(&report.layer),
        boundary_summary(&report.context),
        report.public_surface
    );

    output.push_str("imports:\n");
    for import in &report.imports {
        output.push_str(&format!(
            "- {} {} {}:{}",
            import.resolution, import.specifier, import.line, import.column
        ));
        if let Some(target_file) = &import.target_file {
            output.push_str(&format!(" -> {target_file}"));
        }
        if let Some(package_name) = &import.package_name {
            output.push_str(&format!(" package {package_name}"));
        }
        if let Some(package_allowed) = import.package_allowed {
            output.push_str(&format!(" allowed {package_allowed}"));
        }
        output.push('\n');
    }

    output.push_str("violations:\n");
    for violation in &report.violations {
        output.push_str(&render_pretty_violation(violation, include_tips));
    }

    output
}

fn render_pretty_summary(report: &CheckReport) -> String {
    format!(
        "{} {} ({} {}, {} {})\n{} {}\nstatus: {}\n",
        report.summary.violation_count,
        pluralize(report.summary.violation_count, "problem", "problems"),
        report.summary.error_count,
        pluralize(report.summary.error_count, "error", "errors"),
        report.summary.warning_count,
        pluralize(report.summary.warning_count, "warning", "warnings"),
        report.summary.file_count,
        pluralize(report.summary.file_count, "file checked", "files checked"),
        report.status.as_str()
    )
}

pub fn render_llm(report: &CheckReport) -> String {
    let groups = llm_groups(report);
    let mut output = format!(
        "status: {}\nfilesChecked: {}\nproblemCount: {}\nerrorCount: {}\nwarningCount: {}\ngroupCount: {}\n",
        report.status.as_str(),
        report.summary.file_count,
        report.summary.violation_count,
        report.summary.error_count,
        report.summary.warning_count,
        groups.len()
    );

    for (index, group) in groups.iter().enumerate() {
        output.push_str(&format!(
            "\ngroup {}\ncount: {}\nseverity: {}\nrule: {}\nmessage: {}\nwhy: {}\n",
            index + 1,
            group.violations.len(),
            group.key.severity,
            group.key.rule,
            group.key.message,
            violation_rule_explanation(&group.key.rule)
        ));
        if let Some(import_specifier) = &group.key.import_specifier {
            output.push_str(&format!("import: {import_specifier}\n"));
        }
        if let Some(package_name) = &group.key.package_name {
            output.push_str(&format!("package: {package_name}\n"));
        }
        if let (Some(from_layer), Some(to_layer)) = (&group.key.from_layer, &group.key.to_layer) {
            output.push_str(&format!("layers: {from_layer} -> {to_layer}\n"));
        }
        if let (Some(from_context), Some(to_context)) =
            (&group.key.from_context, &group.key.to_context)
        {
            output.push_str(&format!("contexts: {from_context} -> {to_context}\n"));
        }
        if let Some(target_file) = &group.key.target_file {
            output.push_str(&format!("target: {target_file}\n"));
        }
        if let Some(cycle_path) = &group.key.cycle_path {
            output.push_str(&format!("cycle: {}\n", cycle_path.join(" -> ")));
        }
        if let Some(matched_layers) = &group.key.matched_layers {
            output.push_str(&format!("matchedLayers: {}\n", matched_layers.join(", ")));
        }
        if let Some(matched_contexts) = &group.key.matched_contexts {
            output.push_str(&format!(
                "matchedContexts: {}\n",
                matched_contexts.join(", ")
            ));
        }
        if let Some(suggestion) = &group.key.suggestion {
            output.push_str(&format!("tip: {suggestion}\n"));
        }
        output.push_str("locations:\n");
        for violation in &group.violations {
            output.push_str(&format!(
                "- {}:{}:{}\n",
                violation.file,
                pretty_line(violation),
                pretty_column(violation)
            ));
        }
    }

    output.push('\n');
    output.push_str(LLM_REPORT_SEPARATOR);
    output.push('\n');
    output.push_str(LLM_REPORT_METADATA);
    output.push('\n');

    output
}

struct LlmGroup<'a> {
    key: LlmGroupKey,
    violations: Vec<&'a Violation>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct LlmGroupKey {
    rule: String,
    severity: String,
    message: String,
    import_specifier: Option<String>,
    package_name: Option<String>,
    from_layer: Option<String>,
    to_layer: Option<String>,
    from_context: Option<String>,
    to_context: Option<String>,
    target_file: Option<String>,
    cycle_path: Option<Vec<String>>,
    suggestion: Option<String>,
    matched_layers: Option<Vec<String>>,
    matched_contexts: Option<Vec<String>>,
}

impl LlmGroupKey {
    fn from_violation(violation: &Violation) -> Self {
        Self {
            rule: violation.rule.clone(),
            severity: violation.severity.clone(),
            message: violation.message.clone(),
            import_specifier: violation.import_specifier.clone(),
            package_name: violation.package_name.clone(),
            from_layer: violation.from_layer.clone(),
            to_layer: violation.to_layer.clone(),
            from_context: violation.from_context.clone(),
            to_context: violation.to_context.clone(),
            target_file: violation.target_file.clone(),
            cycle_path: violation.cycle_path.clone(),
            suggestion: violation.suggestion.clone(),
            matched_layers: violation.matched_layers.clone(),
            matched_contexts: violation.matched_contexts.clone(),
        }
    }
}

fn llm_groups(report: &CheckReport) -> Vec<LlmGroup<'_>> {
    let mut grouped: BTreeMap<LlmGroupKey, Vec<&Violation>> = BTreeMap::new();
    for violation in &report.violations {
        grouped
            .entry(LlmGroupKey::from_violation(violation))
            .or_default()
            .push(violation);
    }

    let mut groups = grouped
        .into_iter()
        .map(|(key, mut violations)| {
            violations.sort_by(|left, right| {
                left.file
                    .cmp(&right.file)
                    .then_with(|| pretty_line(left).cmp(&pretty_line(right)))
                    .then_with(|| pretty_column(left).cmp(&pretty_column(right)))
            });
            LlmGroup { key, violations }
        })
        .collect::<Vec<_>>();

    groups.sort_by(|left, right| {
        severity_rank(&left.key.severity)
            .cmp(&severity_rank(&right.key.severity))
            .then_with(|| right.violations.len().cmp(&left.violations.len()))
            .then_with(|| left.key.rule.cmp(&right.key.rule))
            .then_with(|| left.key.message.cmp(&right.key.message))
    });
    groups
}

fn severity_rank(severity: &str) -> usize {
    match severity {
        "error" => 0,
        "warn" => 1,
        _ => 2,
    }
}

fn sorted_violations(violations: &[Violation]) -> Vec<&Violation> {
    let mut sorted = violations.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then_with(|| pretty_line(left).cmp(&pretty_line(right)))
            .then_with(|| pretty_column(left).cmp(&pretty_column(right)))
            .then_with(|| left.rule.cmp(&right.rule))
    });
    sorted
}

fn render_pretty_violation(violation: &Violation, include_tips: bool) -> String {
    let mut output = format!(
        "  {}:{}  {:<7} {}  {}\n",
        pretty_line(violation),
        pretty_column(violation),
        pretty_severity(&violation.severity),
        violation.message,
        violation.rule
    );

    if !include_tips {
        return output;
    }

    output.push_str(&format!(
        "    why: {}\n",
        violation_rule_explanation(&violation.rule)
    ));
    if let Some(import_specifier) = &violation.import_specifier {
        output.push_str(&format!("    import: {import_specifier}\n"));
    }
    if let Some(package_name) = &violation.package_name {
        output.push_str(&format!("    package: {package_name}\n"));
    }
    if let (Some(from_layer), Some(to_layer)) = (&violation.from_layer, &violation.to_layer) {
        output.push_str(&format!("    layers: {from_layer} -> {to_layer}\n"));
    }
    if let (Some(from_context), Some(to_context)) = (&violation.from_context, &violation.to_context)
    {
        output.push_str(&format!("    contexts: {from_context} -> {to_context}\n"));
    }
    if let Some(target_file) = &violation.target_file {
        output.push_str(&format!("    target: {target_file}\n"));
    }
    if let Some(cycle_path) = &violation.cycle_path {
        output.push_str(&format!("    cycle: {}\n", cycle_path.join(" -> ")));
    }
    if let Some(matched_layers) = &violation.matched_layers {
        output.push_str(&format!(
            "    matched layers: {}\n",
            matched_layers.join(", ")
        ));
    }
    if let Some(matched_contexts) = &violation.matched_contexts {
        output.push_str(&format!(
            "    matched contexts: {}\n",
            matched_contexts.join(", ")
        ));
    }
    if let Some(suggestion) = &violation.suggestion {
        output.push_str(&format!("    tip: {suggestion}\n"));
    }

    output
}

fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

fn pretty_line(violation: &Violation) -> usize {
    violation.line.unwrap_or(1)
}

fn pretty_column(violation: &Violation) -> usize {
    violation.column.unwrap_or(1)
}

fn pretty_severity(severity: &str) -> &str {
    match severity {
        "warn" => "warning",
        other => other,
    }
}

fn violation_rule_explanation(rule: &str) -> &'static str {
    match rule {
        RULE_UNCLASSIFIED_FILE => {
            "Layer checks need each analyzed file to match exactly one configured architectural layer."
        }
        RULE_AMBIGUOUS_LAYER => {
            "Overlapping layer patterns make it unclear which dependency policy applies to this file."
        }
        RULE_AMBIGUOUS_CONTEXT => {
            "Overlapping context patterns make it unclear which ownership boundary applies to this file."
        }
        RULE_NO_LAYER_LEAK => {
            "Layer rules only allow imports declared in the importing layer's mayImport policy."
        }
        RULE_NO_FORBIDDEN_IMPORTS => {
            "External packages are closed by default in sensitive layers unless explicitly allowed."
        }
        RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT => {
            "Cross-context imports must target the imported context's configured public surface."
        }
        RULE_NO_FRAMEWORK_IN_CORE => "Core layers should depend on ports, not framework packages.",
        RULE_NO_OUTER_DATA_FORMAT_IN_CORE => {
            "Core layers should not mention data formats owned by outer details."
        }
        RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT => {
            "A public surface should expose intentional contracts, not internal implementation files."
        }
        RULE_NO_CONTEXT_CYCLE => {
            "Context dependencies should form a directed acyclic ownership graph."
        }
        RULE_NO_UNOWNED_SCHEMA_IMPORT => {
            "A context should not depend directly on another context's storage schema."
        }
        RULE_CLEAN_ARTIFACT_PLACEMENT => {
            "Clean Architecture artifacts should live under a context-first layer boundary or a contextless base layer."
        }
        RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT => {
            "Cross-slice imports should target the imported slice's configured public surface."
        }
        RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS => {
            "Vertical Slice artifacts should live under the configured slice root unless their global folder is explicitly allowed."
        }
        RULE_VERTICAL_SLICE_ENTRY_POINT => {
            "Each Vertical Slice should expose a small configured entry point so routes, jobs, or composition code depend on the slice boundary."
        }
        RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS => {
            "Vertical Slice projects should not rebuild shared technical layers such as global services, repositories, handlers, or use cases."
        }
        RULE_NO_CONCRETE_DEPENDENCY => {
            "Core layers should depend on abstractions rather than concrete details."
        }
        RULE_FEATURE_ENVY => {
            "A file that mostly imports another context may contain behavior owned by that context."
        }
        RULE_SHOTGUN_SURGERY => {
            "Files that repeatedly change with many companions may hide scattered responsibilities."
        }
        RULE_TEST_PLACEMENT => {
            "Source-level unit tests should live in colocated test directories, while integration and e2e tests should live under their dedicated workspace roots."
        }
        RULE_PATH_NAMING => {
            "Path naming checks observable file and directory names, not code symbols."
        }
        RULE_FEATURE_SYSTEM_LAYOUT => {
            "Feature system layout checks observable systems/<domain> folders, shared UI roots, and surface CSS placement."
        }
        RULE_FEATURE_SYSTEM_PUBLIC_API => {
            "Feature system public APIs should be explicit barrels, and callers outside a system should depend on those barrels instead of internals."
        }
        RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW => {
            "Feature system dependency flow keeps upper UI layers from shortcutting into adapters and keeps routes on public barrels."
        }
        RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT => {
            "Feature system adapter contracts check domain-named API adapters, typed API errors, cancellable reads, and adapter import boundaries."
        }
        RULE_FEATURE_SYSTEM_QUERY_CONTRACT => {
            "Feature system query contracts keep TanStack Query keys, options, hooks, and cache mutations owned by the system lib layer."
        }
        _ => "This finding violates the configured OnionCry architecture policy.",
    }
}

fn boundary_summary(boundary: &BoundaryExplanation) -> String {
    match &boundary.name {
        Some(name) => format!("{} {}", boundary.status, name),
        None => boundary.status.clone(),
    }
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

impl LoadedConfig {
    pub fn project_root(&self) -> Result<PathBuf> {
        let root = normalize_path(&resolve_against(
            &self.config_dir,
            Path::new(&self.config.project.root),
        ));
        if root.is_dir() {
            Ok(root)
        } else {
            Err(OnionCryError::MissingProjectRoot { path: root })
        }
    }

    fn alias_mappings(&self) -> Vec<AliasMapping> {
        let mut aliases = self
            .config
            .aliases
            .iter()
            .filter_map(|(prefix, target)| {
                target.as_str().map(|target| AliasMapping {
                    prefix: prefix.to_string(),
                    target: PathBuf::from(target),
                })
            })
            .collect::<Vec<_>>();
        aliases.sort_by_key(|alias| std::cmp::Reverse(alias.prefix.len()));
        aliases
    }
}

impl Violation {
    fn unclassified_file(file: &Path, severity: Severity) -> Self {
        Self {
            rule: RULE_UNCLASSIFIED_FILE.to_string(),
            severity: severity.as_str().to_string(),
            message: "file is not classified by any configured architectural layer".to_string(),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "add a matching layers.*.patterns entry or exclude the file".to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn ambiguous_layer(file: &Path, matched_layers: Vec<String>, severity: Severity) -> Self {
        Self {
            rule: RULE_AMBIGUOUS_LAYER.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "file matches multiple architectural layers: {}",
                matched_layers.join(", ")
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some("make layer patterns mutually exclusive".to_string()),
            matched_layers: Some(matched_layers),
            matched_contexts: None,
        }
    }

    fn layer_leak(
        edge: &ImportEdge,
        target: &Path,
        from_layer: &str,
        to_layer: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_LAYER_LEAK.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not import {to_layer} through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: Some(to_layer.to_string()),
            from_context: None,
            to_context: None,
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(format!(
                "add {to_layer:?} to layers.{from_layer}.mayImport only if this dependency is intentional"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn forbidden_external_package(
        edge: &ImportEdge,
        from_layer: &str,
        package_name: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not import external package {package_name} through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: Some(package_name.to_string()),
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "add {package_name:?} to the cleanarch/no-forbidden-imports allowlist for {from_layer} only if this package is domain-safe for that layer"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn framework_in_core(
        edge: &ImportEdge,
        from_layer: &str,
        package_name: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_FRAMEWORK_IN_CORE.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not depend on framework package {package_name} through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: Some(package_name.to_string()),
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "depend on a core-owned port and move framework code to an outer layer".to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn outer_data_format_in_core(
        edge: &ImportEdge,
        target: &Path,
        from_layer: &str,
        to_layer: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_OUTER_DATA_FORMAT_IN_CORE.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not import {to_layer} data format through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: Some(to_layer.to_string()),
            from_context: None,
            to_context: None,
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "define a core-owned type or mapper instead of importing outer data formats"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn ambiguous_context(file: &Path, matched_contexts: Vec<String>, severity: Severity) -> Self {
        Self {
            rule: RULE_AMBIGUOUS_CONTEXT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "file matches multiple architectural contexts: {}",
                matched_contexts.join(", ")
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some("make context patterns mutually exclusive".to_string()),
            matched_layers: None,
            matched_contexts: Some(matched_contexts),
        }
    }

    fn cross_context_internal_import(
        edge: &ImportEdge,
        target: &Path,
        from_context: &str,
        to_context: &str,
        severity: Severity,
        public_surface_segments: &HashSet<String>,
    ) -> Self {
        let mut segments = public_surface_segments.iter().cloned().collect::<Vec<_>>();
        segments.sort();
        Self {
            rule: RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_context} may not import {to_context} internal details through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: Some(from_context.to_string()),
            to_context: Some(to_context.to_string()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(format!(
                "import from the {to_context} public surface segment instead{}",
                if segments.is_empty() {
                    String::new()
                } else {
                    format!(": {}", segments.join(", "))
                }
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn public_surface_internal_reexport(
        edge: &ImportEdge,
        target: &Path,
        context: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{context} public surface may not re-export internal detail through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: Some(context.to_string()),
            to_context: Some(context.to_string()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "move the contract into the public surface or stop re-exporting it".to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn context_cycle(
        edge: &ImportEdge,
        target: &Path,
        context_path: &[String],
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_CONTEXT_CYCLE.to_string(),
            severity: severity.as_str().to_string(),
            message: format!("context dependency cycle: {}", context_path.join(" -> ")),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: context_path.first().cloned(),
            to_context: context_path.get(1).cloned(),
            target_file: Some(target.display().to_string()),
            cycle_path: Some(context_path.to_vec()),
            suggestion: Some(
                "extract a public contract or shared kernel so context dependencies point one way"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn unowned_schema_import(
        edge: &ImportEdge,
        target: &Path,
        from_context: &str,
        to_context: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_UNOWNED_SCHEMA_IMPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_context} may not import {to_context} owned schema through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: Some(from_context.to_string()),
            to_context: Some(to_context.to_string()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "depend on the owning context contract instead of importing its storage schema"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn concrete_dependency(
        edge: &ImportEdge,
        target: Option<&Path>,
        from_layer: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_CONCRETE_DEPENDENCY.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not depend on concrete detail through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: target.map(|target| target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "depend on an abstraction owned by the core layer and bind the concrete detail outside"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn feature_envy(
        file: &Path,
        from_context: &str,
        to_context: &str,
        import_count: usize,
        own_context_count: usize,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_ENVY.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_context} file imports {import_count} dependencies from {to_context} and {own_context_count} from its own context"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: Some(from_context.to_string()),
            to_context: Some(to_context.to_string()),
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "move the behavior closer to the context it uses or depend on a smaller public contract"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn clean_artifact_placement(
        file: &Path,
        severity: Severity,
        role: &str,
        expected_layer: &str,
        expected_boundary: &str,
    ) -> Self {
        Self {
            rule: RULE_CLEAN_ARTIFACT_PLACEMENT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "cleanArchitecture artifact {role:?} should live in the {expected_layer} boundary {expected_boundary}"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: Some(expected_layer.to_string()),
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "move this {role} artifact to {expected_boundary} or turn cleanarch/artifact-placement off with an override while migrating"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn cross_slice_internal_import(
        edge: &ImportEdge,
        target: &Path,
        severity: Severity,
        source_location: &SliceLocation,
        target_location: &SliceLocation,
    ) -> Self {
        Self {
            rule: RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "verticalSlice slice {} may not import {} internals through {}",
                source_location.slice, target_location.slice, edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: Some(source_location.slice.clone()),
            to_context: Some(target_location.slice.clone()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(format!(
                "import from the {} public surface instead, such as index.ts or contracts/",
                target_location.slice_path
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn global_slice_artifact(
        file: &Path,
        severity: Severity,
        role: &str,
        slice_root_pattern: &str,
    ) -> Self {
        Self {
            rule: RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "verticalSlice artifact {role:?} is outside the configured slice layout {slice_root_pattern}"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: Some(role.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "move this slice artifact under {slice_root_pattern} or add its global folder to architecture.verticalSlice.allowedGlobalFolders"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn vertical_slice_entry_point(
        file: &Path,
        severity: Severity,
        slice: &str,
        slice_path: &str,
        entry_point_names: &str,
    ) -> Self {
        Self {
            rule: RULE_VERTICAL_SLICE_ENTRY_POINT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "verticalSlice slice {slice} does not expose a configured entry point"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: Some(slice.to_string()),
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "export one entry point from {slice_path}, such as one of {entry_point_names}"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn vertical_shared_layer_artifact(
        file: &Path,
        severity: Severity,
        folder: &str,
        slice_root_pattern: &str,
    ) -> Self {
        Self {
            rule: RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "verticalSlice shared layer folder {folder:?} is outside the configured slice layout"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: Some(folder.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "move this artifact under {slice_root_pattern} or keep only cross-cutting platform code in configured global folders"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn shotgun_surgery(
        file: &Path,
        commit_count: usize,
        related_file_count: usize,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_SHOTGUN_SURGERY.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "file changed in {commit_count} commits with {related_file_count} recurring companion files"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "look for scattered responsibilities and extract a boundary that changes together"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn misplaced_test_file(file: &Path, severity: Severity, suggestion: String) -> Self {
        Self {
            rule: RULE_TEST_PLACEMENT.to_string(),
            severity: severity.as_str().to_string(),
            message: "test file is not in an allowed test location".to_string(),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn path_naming(file: &Path, severity: Severity, message: String, suggestion: String) -> Self {
        Self {
            rule: RULE_PATH_NAMING.to_string(),
            severity: severity.as_str().to_string(),
            message,
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn feature_system_layout(
        file: &Path,
        severity: Severity,
        message: String,
        suggestion: String,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_LAYOUT.to_string(),
            severity: severity.as_str().to_string(),
            message,
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn feature_system_public_api_wildcard(file: &Path, severity: Severity) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_PUBLIC_API.to_string(),
            severity: severity.as_str().to_string(),
            message: "feature system public API must not use wildcard re-exports".to_string(),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "replace wildcard re-exports with explicit named exports from the system barrel"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn feature_system_public_api_internal_import(
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
        }
    }

    fn feature_system_dependency_flow(
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
        }
    }

    fn feature_system_adapter_contract(
        file: &Path,
        severity: Severity,
        message: String,
        suggestion: String,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT.to_string(),
            severity: severity.as_str().to_string(),
            message,
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    fn feature_system_adapter_contract_import(
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
        }
    }

    fn feature_system_query_contract(
        file: &Path,
        severity: Severity,
        message: String,
        suggestion: String,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_QUERY_CONTRACT.to_string(),
            severity: severity.as_str().to_string(),
            message,
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
        }
    }
}

impl RulePolicy {
    fn new(config: &Config) -> Result<Self> {
        let base_rules = parse_rule_map(&config.rules)?;
        validate_architecture_rule_mode(config.architecture.mode, &base_rules)?;
        let mut overrides = Vec::new();

        for override_config in &config.overrides {
            let rules = parse_rule_map(&override_config.rules)?;
            validate_architecture_rule_mode(config.architecture.mode, &rules)?;
            overrides.push(CompiledOverride {
                files: build_glob_set(&override_config.files)?,
                rules,
            });
        }

        Ok(Self {
            base_rules,
            overrides,
        })
    }

    fn effective_severity(&self, rule: &str, project_root: &Path, file: &Path) -> Severity {
        self.effective_rule(rule, project_root, file).severity
    }

    fn effective_rule(&self, rule: &str, project_root: &Path, file: &Path) -> RuleSetting {
        let relative_path = file.strip_prefix(project_root).unwrap_or(file);
        let mut setting = self
            .base_rules
            .get(rule)
            .cloned()
            .unwrap_or_else(|| default_rule_setting(rule));

        for override_config in &self.overrides {
            if !override_config.files.is_match(relative_path) {
                continue;
            }
            if let Some(override_setting) = override_config.rules.get(rule) {
                setting = override_setting.clone();
            }
        }

        setting
    }
}

impl ExternalPackagePolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let mut policy = Self {
            default_severity: setting.severity,
            default_allow: PackageAllowlist::empty()?,
            layers: Vec::new(),
        };

        let Some(options) = &setting.options else {
            return Ok(policy);
        };

        match options {
            Value::Object(options) => {
                if let Some(allow) = options.get("allow") {
                    policy.default_allow = PackageAllowlist::from_value(allow)?;
                }
                if let Some(layers) = options.get("layers") {
                    policy.layers =
                        parse_external_package_layer_policies(layers, setting.severity)?;
                }
            }
            Value::Array(_) => {
                policy.layers = parse_external_package_layer_policies(options, setting.severity)?;
            }
            _ => {
                return Err(OnionCryError::InvalidRuleValue {
                    rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
                    message: "expected options object or layer policy array".to_string(),
                });
            }
        }

        Ok(policy)
    }

    fn for_layer(&self, from_layer: &str) -> EffectiveExternalPackageLayerPolicy<'_> {
        let mut effective = EffectiveExternalPackageLayerPolicy {
            severity: self.default_severity,
            allow: &self.default_allow,
        };

        for layer in &self.layers {
            if layer.from_layer == from_layer {
                effective = EffectiveExternalPackageLayerPolicy {
                    severity: layer.severity,
                    allow: &layer.allow,
                };
            }
        }

        effective
    }
}

impl PackageAllowlist {
    fn empty() -> Result<Self> {
        Self::from_patterns(&[])
    }

    fn from_value(value: &Value) -> Result<Self> {
        let patterns = match value {
            Value::String(pattern) => vec![pattern.clone()],
            Value::Array(items) => items
                .iter()
                .map(|item| {
                    item.as_str().map(str::to_string).ok_or_else(|| {
                        OnionCryError::InvalidRuleValue {
                            rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
                            message: "allow entries must be package pattern strings".to_string(),
                        }
                    })
                })
                .collect::<Result<Vec<_>>>()?,
            _ => {
                return Err(OnionCryError::InvalidRuleValue {
                    rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
                    message: "allow must be a package pattern string or array of strings"
                        .to_string(),
                });
            }
        };

        Self::from_patterns(&patterns)
    }

    fn from_patterns(patterns: &[String]) -> Result<Self> {
        Ok(Self {
            patterns: build_glob_set(patterns)?,
        })
    }

    fn is_allowed(&self, package_name: &str) -> bool {
        self.patterns.is_match(package_name)
    }
}

impl TestPlacementPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let source_roots =
            string_vec_option(RULE_TEST_PLACEMENT, setting, "sourceRoots", &["src"])?;
        let unit_test_directories = string_set_option(
            RULE_TEST_PLACEMENT,
            setting,
            "unitTestDirectories",
            &["__tests__"],
        )?;
        let integration_roots = string_vec_option(
            RULE_TEST_PLACEMENT,
            setting,
            "integrationRoots",
            &["tests/integration"],
        )?;
        let e2e_roots =
            string_vec_option(RULE_TEST_PLACEMENT, setting, "e2eRoots", &["tests/e2e"])?;
        let test_file_suffixes = string_vec_option(
            RULE_TEST_PLACEMENT,
            setting,
            "testFileSuffixes",
            DEFAULT_TEST_FILE_SUFFIXES,
        )?
        .into_iter()
        .map(|suffix| suffix.to_ascii_lowercase())
        .collect();

        Ok(Self {
            source_roots: path_roots(source_roots),
            unit_test_directories,
            integration_roots: path_roots(integration_roots),
            e2e_roots: path_roots(e2e_roots),
            test_file_suffixes,
        })
    }

    fn is_test_file(&self, project_root: &Path, file: &Path) -> bool {
        let relative_path = project_relative_display(project_root, file).to_ascii_lowercase();
        self.test_file_suffixes
            .iter()
            .any(|suffix| relative_path.ends_with(suffix))
    }

    fn is_valid_test_placement(&self, project_root: &Path, file: &Path) -> bool {
        let components = project_relative_components(project_root, file);
        self.is_under_context_root(&components, &self.integration_roots)
            || self.is_under_context_root(&components, &self.e2e_roots)
            || (self.is_under_any_root(&components, &self.source_roots)
                && components
                    .iter()
                    .any(|segment| self.unit_test_directories.contains(segment)))
    }

    fn suggestion(&self, project_root: &Path, file: &Path) -> String {
        let components = project_relative_components(project_root, file);
        if self.is_under_any_root(&components, &self.integration_roots) {
            return format!(
                "place integration tests under {}/<context>/",
                display_root(&self.integration_roots)
            );
        }
        if self.is_under_any_root(&components, &self.e2e_roots) {
            return format!(
                "place e2e tests under {}/<context>/",
                display_root(&self.e2e_roots)
            );
        }
        if self.is_under_any_root(&components, &self.source_roots) {
            return format!(
                "move this unit test into a colocated {} directory",
                display_unit_test_directories(&self.unit_test_directories)
            );
        }

        format!(
            "move this test under {}, {}, or a colocated {} directory in {}",
            display_root(&self.integration_roots),
            display_root(&self.e2e_roots),
            display_unit_test_directories(&self.unit_test_directories),
            display_root(&self.source_roots)
        )
    }

    fn is_under_context_root(&self, components: &[String], roots: &[Vec<String>]) -> bool {
        roots.iter().any(|root| {
            path_has_prefix_components(components, root) && components.len() >= root.len() + 2
        })
    }

    fn is_under_any_root(&self, components: &[String], roots: &[Vec<String>]) -> bool {
        roots
            .iter()
            .any(|root| path_has_prefix_components(components, root))
    }
}

impl PathNamingPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let collection_directories = string_set_option(
            RULE_PATH_NAMING,
            setting,
            "collectionDirectories",
            DEFAULT_COLLECTION_DIRECTORIES,
        )?;
        let feature_roots = string_vec_option(
            RULE_PATH_NAMING,
            setting,
            "featureRoots",
            DEFAULT_FEATURE_ROOTS,
        )?;
        let layer_directories = string_set_option(
            RULE_PATH_NAMING,
            setting,
            "layerDirectories",
            DEFAULT_LAYER_DIRECTORIES,
        )?;
        let ignored_directories = string_set_option(
            RULE_PATH_NAMING,
            setting,
            "ignoredDirectories",
            DEFAULT_IGNORED_PATH_DIRECTORIES,
        )?;
        let suffixes_by_collection = suffix_map_option(
            RULE_PATH_NAMING,
            setting,
            "suffixes",
            DEFAULT_SUFFIXES_BY_COLLECTION,
        )?;
        let singular_collection_directories = collection_directories
            .iter()
            .map(|directory| (singular_directory_name(directory), directory.clone()))
            .filter(|(singular, plural)| singular != plural)
            .collect();

        Ok(Self {
            collection_directories,
            singular_collection_directories,
            feature_roots: path_roots(feature_roots),
            layer_directories,
            ignored_directories,
            suffixes_by_collection,
        })
    }

    fn violations(&self, project_root: &Path, file: &Path, severity: Severity) -> Vec<Violation> {
        let components = project_relative_components(project_root, file);
        let mut violations = Vec::new();

        for (index, directory) in components
            .iter()
            .take(components.len().saturating_sub(1))
            .enumerate()
        {
            if self.ignored_directories.contains(directory) {
                continue;
            }
            if !is_kebab_case_name(directory) {
                violations.push(Violation::path_naming(
                    file,
                    severity,
                    format!("directory segment {directory:?} should use kebab-case"),
                    "rename the directory segment to lowercase kebab-case".to_string(),
                ));
            }
            if let Some(plural) = self.singular_collection_directories.get(directory) {
                violations.push(Violation::path_naming(
                    file,
                    severity,
                    format!("collection directory {directory:?} should be plural"),
                    format!("rename {directory:?} to {plural:?}"),
                ));
            }
            if directory == "infrastructure" && !self.layer_directories.contains(directory) {
                violations.push(Violation::path_naming(
                    file,
                    severity,
                    "layer directory \"infrastructure\" should use the configured layer vocabulary"
                        .to_string(),
                    "use \"infra\" or configure repo/path-naming.layerDirectories".to_string(),
                ));
            }
            if self.is_feature_segment(&components, index) && plural_like(directory) {
                violations.push(Violation::path_naming(
                    file,
                    severity,
                    format!("feature directory {directory:?} should be singular"),
                    format!(
                        "rename {directory:?} to a singular kebab-case feature or context name"
                    ),
                ));
            }
        }

        if let Some(file_name) = components.last() {
            if !is_kebab_case_file_name(file_name) {
                violations.push(Violation::path_naming(
                    file,
                    severity,
                    format!("file name {file_name:?} should use kebab-case"),
                    "rename the file to lowercase kebab-case while keeping any configured suffix"
                        .to_string(),
                ));
            }
            if let Some((collection, suffixes)) = self.nearest_suffix_collection(&components) {
                let stem = source_file_stem(file_name);
                if stem != "index" && !stem_matches_collection_suffix(stem, suffixes) {
                    violations.push(Violation::path_naming(
                        file,
                        severity,
                        format!("files in {collection:?} should use a configured suffix"),
                        format!(
                            "rename the file so its stem ends with one of: {} (optionally followed by .test or .spec)",
                            suffixes.join(", ")
                        ),
                    ));
                }
            }
        }

        violations
    }

    fn is_feature_segment(&self, components: &[String], index: usize) -> bool {
        self.feature_roots.iter().any(|root| {
            index == root.len()
                && path_has_prefix_components(components, root)
                && !self.collection_directories.contains(&components[index])
                && !self.layer_directories.contains(&components[index])
                && !self.ignored_directories.contains(&components[index])
        })
    }

    fn nearest_suffix_collection<'a>(
        &'a self,
        components: &[String],
    ) -> Option<(&'a str, &'a [String])> {
        components.iter().rev().skip(1).find_map(|component| {
            self.suffixes_by_collection
                .get_key_value(component)
                .map(|(collection, suffixes)| (collection.as_str(), suffixes.as_slice()))
        })
    }
}

impl CleanArtifactPlacementFileIndex {
    fn from_files(project_root: &Path, files: &[PathBuf]) -> Self {
        let mut direct_source_file_counts = BTreeMap::<Vec<String>, usize>::new();

        for file in files {
            let components = project_relative_components(project_root, file);
            let Some(file_name) = components.last() else {
                continue;
            };
            if is_index_file_name(file_name) || is_test_file_name(file_name) {
                continue;
            }
            let Some(parent) = components.get(..components.len().saturating_sub(1)) else {
                continue;
            };
            *direct_source_file_counts
                .entry(parent.to_vec())
                .or_default() += 1;
        }

        Self {
            direct_source_file_counts,
        }
    }

    fn direct_file_count(&self, components: &[String], folder_index: usize) -> usize {
        components
            .get(..=folder_index)
            .and_then(|parent| self.direct_source_file_counts.get(parent))
            .copied()
            .unwrap_or_default()
    }
}

impl CleanArtifactPlacementPolicy {
    fn from_config(config: &CleanArchitectureConfig) -> Self {
        let mut layer_aliases = BTreeMap::new();
        for layer in ["domain", "application", "infra"] {
            let mut aliases = config
                .layer_path_aliases
                .get(layer)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect::<HashSet<_>>();
            aliases.insert(layer.to_string());
            layer_aliases.insert(layer.to_string(), aliases);
        }

        let mut folder_layers = BTreeMap::<String, BTreeSet<String>>::new();
        for (layer, folders) in &config.artifact_folders {
            for folder in folders {
                folder_layers
                    .entry(folder.clone())
                    .or_default()
                    .insert(layer.clone());
            }
        }

        let context_root = path_components(Path::new(&config.context_root));
        let source_prefix = context_root
            .split_last()
            .map(|(_, prefix)| prefix.to_vec())
            .unwrap_or_default();

        Self {
            source_prefix,
            context_root,
            layer_aliases,
            artifact_folders_by_layer: config.artifact_folders.clone(),
            folder_layers,
            suffix_roles: config.artifact_suffixes.clone(),
            grouped_artifact_folders: config.grouped_artifact_folders.iter().cloned().collect(),
        }
    }

    fn finding(
        &self,
        project_root: &Path,
        file: &Path,
        file_index: &CleanArtifactPlacementFileIndex,
    ) -> Option<CleanArtifactFinding> {
        let components = project_relative_components(project_root, file);
        if components.is_empty() {
            return None;
        }
        let artifact = match self.artifact_classification(&components) {
            Some(artifact) => artifact,
            None => return self.layer_direct_folder_finding(&components, file_index),
        };
        let current_layer = self.first_layer(&components);
        let expected_layer = self.expected_layer(&artifact.role, current_layer.as_deref())?;

        if let Some((context, layer)) = self.context_first_location(&components) {
            if layer == expected_layer {
                return None;
            }
            return Some(CleanArtifactFinding {
                role: artifact.role.clone(),
                expected_boundary: self.expected_boundary(
                    Some(&context),
                    &expected_layer,
                    &artifact.role,
                ),
                expected_layer,
            });
        }

        if let Some(context) = self.context_missing_layer(&components) {
            return Some(CleanArtifactFinding {
                role: artifact.role.clone(),
                expected_boundary: self.expected_boundary(
                    Some(&context),
                    &expected_layer,
                    &artifact.role,
                ),
                expected_layer,
            });
        }

        if let Some((layer, layer_index)) = self.first_layer_with_index(&components) {
            if layer != expected_layer {
                let context = self.context_before_layer(&components, layer_index);
                return Some(CleanArtifactFinding {
                    role: artifact.role.clone(),
                    expected_boundary: self.expected_boundary(
                        context,
                        &expected_layer,
                        &artifact.role,
                    ),
                    expected_layer,
                });
            }
            if self.is_contextless_base_layer(&components, layer_index) {
                if self.is_flat_grouped_artifact(&components, layer_index, &artifact, file_index) {
                    return Some(CleanArtifactFinding {
                        role: artifact.role.clone(),
                        expected_boundary: self
                            .grouped_artifact_boundary(&expected_layer, &artifact.role),
                        expected_layer,
                    });
                }
                if let Some(expected_boundary) = self.contextless_base_capability_boundary(
                    &components,
                    layer_index,
                    &artifact,
                    file_index,
                    &expected_layer,
                ) {
                    return Some(CleanArtifactFinding {
                        role: artifact.role.clone(),
                        expected_boundary,
                        expected_layer,
                    });
                }
                if let Some(context) = self.layer_first_context_candidate(
                    &components,
                    layer_index,
                    artifact.folder_index,
                ) {
                    return Some(CleanArtifactFinding {
                        role: artifact.role.clone(),
                        expected_boundary: self.expected_boundary(
                            Some(context),
                            &expected_layer,
                            &artifact.role,
                        ),
                        expected_layer,
                    });
                }
                return None;
            }
            return Some(CleanArtifactFinding {
                role: artifact.role.clone(),
                expected_boundary: self.expected_boundary(
                    self.context_before_layer(&components, layer_index),
                    &expected_layer,
                    &artifact.role,
                ),
                expected_layer,
            });
        }

        let context = self.context_after_source_prefix(&components);
        Some(CleanArtifactFinding {
            role: artifact.role.clone(),
            expected_boundary: self.expected_boundary(context, &expected_layer, &artifact.role),
            expected_layer,
        })
    }

    fn artifact_classification(&self, components: &[String]) -> Option<CleanArtifact> {
        let folder_match = components
            .iter()
            .take(components.len().saturating_sub(1))
            .enumerate()
            .rev()
            .find_map(|(index, component)| {
                self.folder_layers
                    .contains_key(component)
                    .then(|| CleanArtifact {
                        role: component.clone(),
                        folder_index: Some(index),
                    })
            });
        if folder_match.is_some() {
            return folder_match;
        }

        let relative_path = components.join("/").to_ascii_lowercase();
        self.suffix_roles.iter().find_map(|(role, suffixes)| {
            let role_folder = artifact_role_folder(role);
            suffixes
                .iter()
                .any(|suffix| relative_path.ends_with(&suffix.to_ascii_lowercase()))
                .then_some(role_folder)
                .filter(|role_folder| {
                    self.folder_layers.contains_key(role_folder)
                        || is_core_clean_artifact_role(role_folder)
                })
                .map(|role_folder| CleanArtifact {
                    role: role_folder,
                    folder_index: None,
                })
        })
    }

    fn layer_direct_folder_finding(
        &self,
        components: &[String],
        file_index: &CleanArtifactPlacementFileIndex,
    ) -> Option<CleanArtifactFinding> {
        let (layer, layer_index) = self.first_layer_with_index(components)?;
        let child_index = layer_index + 1;
        let child = components.get(child_index)?;
        if is_index_file_name(child) || self.layer_for_segment(child).is_some() {
            return None;
        }
        let folders = self.artifact_folders_by_layer.get(&layer)?;
        if folders.is_empty() || folders.iter().any(|folder| folder == child) {
            return None;
        }

        let context = if self.is_contextless_base_layer(components, layer_index) {
            None
        } else if let Some((context, context_layer)) = self.context_first_location(components) {
            (context_layer == layer).then_some(context)
        } else {
            self.context_before_layer(components, layer_index)
                .map(str::to_string)
        };
        let group =
            (file_index.direct_file_count(components, child_index) > 1).then_some(child.as_str());

        Some(CleanArtifactFinding {
            role: layer.clone(),
            expected_boundary: self.layer_artifact_boundary(context.as_deref(), &layer, group),
            expected_layer: layer,
        })
    }

    fn layer_artifact_boundary(
        &self,
        context: Option<&str>,
        layer: &str,
        group: Option<&str>,
    ) -> String {
        let Some(folders) = self.artifact_folders_by_layer.get(layer) else {
            return self.expected_boundary_with_group(context, layer, "<artifact-folder>", group);
        };

        if folders.is_empty() {
            return self.expected_boundary_with_group(context, layer, "<artifact-folder>", group);
        }

        if folders.len() <= 3 {
            return folders
                .iter()
                .map(|folder| self.expected_boundary_with_group(context, layer, folder, group))
                .collect::<Vec<_>>()
                .join(" or ");
        }

        self.expected_boundary_with_group(context, layer, "<artifact-folder>", group)
    }

    fn expected_layer(&self, role: &str, current_layer: Option<&str>) -> Option<String> {
        let candidates = self.folder_layers.get(role);
        if let (Some(current_layer), Some(candidates)) = (current_layer, candidates)
            && candidates.contains(current_layer)
        {
            return Some(current_layer.to_string());
        }
        if let Some(candidates) = candidates
            && candidates.len() == 1
        {
            return candidates.first().cloned();
        }
        current_layer.map(str::to_string)
    }

    fn context_first_location(&self, components: &[String]) -> Option<(String, String)> {
        let context_index = self.context_root.len();
        let layer_index = context_index + 1;
        if !path_has_prefix_components(components, &self.context_root)
            || components.len() <= layer_index
        {
            return None;
        }
        let layer = self.layer_for_segment(&components[layer_index])?;
        Some((components[context_index].clone(), layer))
    }

    fn context_missing_layer(&self, components: &[String]) -> Option<String> {
        let context_index = self.context_root.len();
        let next_index = context_index + 1;
        if !path_has_prefix_components(components, &self.context_root)
            || components.len() <= next_index
        {
            return None;
        }
        let next = &components[next_index];
        (self.layer_for_segment(next).is_none() && self.folder_layers.contains_key(next))
            .then(|| components[context_index].clone())
    }

    fn first_layer(&self, components: &[String]) -> Option<String> {
        self.first_layer_with_index(components)
            .map(|(layer, _)| layer)
    }

    fn first_layer_with_index(&self, components: &[String]) -> Option<(String, usize)> {
        components
            .iter()
            .enumerate()
            .find_map(|(index, component)| {
                self.layer_for_segment(component)
                    .map(|layer| (layer, index))
            })
    }

    fn layer_for_segment(&self, segment: &str) -> Option<String> {
        self.layer_aliases
            .iter()
            .find_map(|(layer, aliases)| aliases.contains(segment).then(|| layer.clone()))
    }

    fn source_prefix_len(&self, components: &[String]) -> usize {
        if path_has_prefix_components(components, &self.source_prefix) {
            self.source_prefix.len()
        } else {
            0
        }
    }

    fn is_contextless_base_layer(&self, components: &[String], layer_index: usize) -> bool {
        layer_index == self.source_prefix_len(components)
    }

    fn context_before_layer<'a>(
        &self,
        components: &'a [String],
        layer_index: usize,
    ) -> Option<&'a str> {
        let context_index = self.source_prefix_len(components);
        (layer_index > context_index)
            .then(|| components.get(context_index).map(String::as_str))
            .flatten()
    }

    fn context_after_source_prefix<'a>(&self, components: &'a [String]) -> Option<&'a str> {
        components
            .get(self.source_prefix_len(components))
            .map(String::as_str)
    }

    fn layer_first_context_candidate<'a>(
        &self,
        components: &'a [String],
        layer_index: usize,
        folder_index: Option<usize>,
    ) -> Option<&'a str> {
        if folder_index == Some(layer_index + 1) {
            return None;
        }
        let context_index = layer_index + 1;
        if components.len() <= context_index + 1 {
            return None;
        }
        let candidate = components[context_index].as_str();
        if self.layer_for_segment(candidate).is_some() || self.folder_layers.contains_key(candidate)
        {
            return None;
        }
        Some(candidate)
    }

    fn is_flat_grouped_artifact(
        &self,
        components: &[String],
        layer_index: usize,
        artifact: &CleanArtifact,
        file_index: &CleanArtifactPlacementFileIndex,
    ) -> bool {
        let Some(folder_index) = artifact.folder_index else {
            return false;
        };
        self.grouped_artifact_folders.contains(&artifact.role)
            && folder_index == layer_index + 1
            && components.len() == folder_index + 2
            && file_index.direct_file_count(components, folder_index) > 1
            && components
                .last()
                .is_some_and(|file_name| !is_index_file_name(file_name))
    }

    fn contextless_base_capability_boundary(
        &self,
        components: &[String],
        layer_index: usize,
        artifact: &CleanArtifact,
        file_index: &CleanArtifactPlacementFileIndex,
        expected_layer: &str,
    ) -> Option<String> {
        let child_index = layer_index + 1;
        let child = components.get(child_index)?;
        if artifact.folder_index == Some(child_index)
            || self.layer_for_segment(child).is_some()
            || self.folder_layers.contains_key(child)
            || is_index_file_name(child)
        {
            return None;
        }

        let group =
            (file_index.direct_file_count(components, child_index) > 1).then_some(child.as_str());
        Some(self.expected_boundary_with_group(None, expected_layer, &artifact.role, group))
    }

    fn grouped_artifact_boundary(&self, layer: &str, role: &str) -> String {
        format!("{}/<group>", self.expected_boundary(None, layer, role))
    }

    fn expected_boundary(&self, context: Option<&str>, layer: &str, role: &str) -> String {
        self.expected_boundary_with_group(context, layer, role, None)
    }

    fn expected_boundary_with_group(
        &self,
        context: Option<&str>,
        layer: &str,
        role: &str,
        group: Option<&str>,
    ) -> String {
        let mut components = Vec::new();
        if let Some(context) = context {
            components.extend(self.context_root.clone());
            components.push(context.to_string());
        } else {
            components.extend(self.source_prefix.clone());
        }
        components.push(layer.to_string());
        components.push(role.to_string());
        if let Some(group) = group {
            components.push(group.to_string());
        }
        display_path_components(&components)
    }
}

struct CleanArtifact {
    role: String,
    folder_index: Option<usize>,
}

impl VerticalSlicePolicy {
    fn from_config(config: &VerticalSliceConfig) -> Self {
        Self {
            slice_root: path_components(Path::new(&config.slice_root)),
            slice_depth: config.slice_depth.max(1),
            public_surface: config
                .public_surface
                .iter()
                .map(|entry| path_components(Path::new(entry)))
                .collect(),
            artifact_folders: config.artifact_folders.iter().cloned().collect(),
            artifact_suffixes: config
                .artifact_suffixes
                .iter()
                .flat_map(|(role, suffixes)| {
                    suffixes
                        .iter()
                        .map(|suffix| (artifact_role_folder(role), suffix.clone()))
                        .collect::<Vec<_>>()
                })
                .collect(),
            allowed_global_folders: config.allowed_global_folders.iter().cloned().collect(),
            entry_point_names: config.entry_point_names.clone(),
            shared_layer_folders: config.shared_layer_folders.iter().cloned().collect(),
        }
    }

    fn slice_location(&self, project_root: &Path, file: &Path) -> Option<SliceLocation> {
        let components = project_relative_components(project_root, file);
        if components.is_empty() {
            return None;
        }
        if self.slice_root.is_empty() {
            if self.allowed_global_folders.contains(&components[0])
                || components.len() <= self.slice_depth
            {
                return None;
            }
            let slice_end = self.slice_depth;
            return Some(SliceLocation {
                slice: display_path_components(&components[..slice_end]),
                slice_path: display_path_components(&components[..slice_end]),
                relative_file: components[slice_end..].to_vec(),
            });
        }

        if components.len() <= self.slice_root.len()
            || !path_has_prefix_components(&components, &self.slice_root)
        {
            return None;
        }
        let slice_index = self.slice_root.len();
        let slice_end = slice_index + self.slice_depth;
        if components.len() <= slice_end {
            return None;
        }
        let slice = display_path_components(&components[slice_index..slice_end]);
        let slice_path = display_path_components(&components[..slice_end]);
        Some(SliceLocation {
            slice,
            slice_path,
            relative_file: components[slice_end..].to_vec(),
        })
    }

    fn is_public_surface_location(&self, location: &SliceLocation) -> bool {
        let relative_file = &location.relative_file;
        self.public_surface.iter().any(|surface| {
            relative_file == surface
                || (!surface.is_empty()
                    && path_has_prefix_components(relative_file.as_slice(), surface)
                    && relative_file.len() > surface.len())
        })
    }

    fn is_allowed_global_file(&self, project_root: &Path, file: &Path) -> bool {
        let components = project_relative_components(project_root, file);
        components
            .first()
            .is_some_and(|segment| self.allowed_global_folders.contains(segment))
    }

    fn slice_artifact_role(&self, project_root: &Path, file: &Path) -> Option<String> {
        let components = project_relative_components(project_root, file);
        let relative_path = components.join("/").to_ascii_lowercase();
        for (role, suffix) in &self.artifact_suffixes {
            if relative_path.ends_with(&suffix.to_ascii_lowercase()) {
                return Some(role.clone());
            }
        }
        components
            .iter()
            .take(components.len().saturating_sub(1))
            .find(|component| {
                component.as_str() != "domain" && self.artifact_folders.contains(*component)
            })
            .cloned()
    }

    fn shared_layer_folder(&self, project_root: &Path, file: &Path) -> Option<String> {
        let components = project_relative_components(project_root, file);
        components
            .iter()
            .take(components.len().saturating_sub(1))
            .find(|component| self.shared_layer_folders.contains(*component))
            .cloned()
    }

    fn entry_point_names_display(&self) -> String {
        self.entry_point_names.join(", ")
    }

    fn slice_root_pattern_display(&self) -> String {
        let mut components = self.slice_root.clone();
        for placeholder_index in 0..self.slice_depth {
            let placeholder = if self.slice_depth == 1 {
                "<feature>"
            } else {
                slice_depth_placeholder(placeholder_index)
            };
            components.push(placeholder.to_string());
        }
        display_path_components(&components)
    }
}

fn slice_depth_placeholder(index: usize) -> &'static str {
    match index {
        0 => "<domain>",
        1 => "<operation>",
        _ => "<segment>",
    }
}

impl FeatureSystemLayoutPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let systems_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "systemsRoots",
            DEFAULT_SYSTEMS_ROOTS,
        )?;
        let required_directories = string_set_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "requiredDirectories",
            DEFAULT_FEATURE_SYSTEM_REQUIRED_DIRECTORIES,
        )?;
        let optional_directories = string_set_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "optionalDirectories",
            DEFAULT_FEATURE_SYSTEM_OPTIONAL_DIRECTORIES,
        )?;
        let root_index_file = string_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "rootIndexFile",
            DEFAULT_FEATURE_SYSTEM_ROOT_INDEX_FILE,
        )?;
        let allowed_shared_component_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "allowedSharedComponentRoots",
            DEFAULT_ALLOWED_SHARED_COMPONENT_ROOTS,
        )?;
        let legacy_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "legacyRoots",
            DEFAULT_LEGACY_FEATURE_ROOTS,
        )?;
        let component_directories = string_set_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "componentDirectories",
            DEFAULT_COMPONENT_DIRECTORIES,
        )?;
        let surface_css_name_template = string_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "surfaceCssNameTemplate",
            DEFAULT_SURFACE_CSS_NAME_TEMPLATE,
        )?;

        Ok(Self {
            systems_roots: path_roots(systems_roots),
            required_directories,
            optional_directories,
            root_index_file,
            allowed_shared_component_roots: path_roots(allowed_shared_component_roots),
            legacy_roots: path_roots(legacy_roots),
            component_directories,
            surface_css_name_template,
        })
    }

    fn violations(
        &self,
        project_root: &Path,
        files: &[PathBuf],
        severity: Severity,
    ) -> Vec<Violation> {
        let systems = self.discover_systems(project_root, files);
        let _recognized_optional_directories = &self.optional_directories;
        let mut violations = Vec::new();

        for system in systems.values() {
            for directory in sorted_strings(&self.required_directories) {
                if !system.path.join(&directory).is_dir() {
                    violations.push(Violation::feature_system_layout(
                        &system.representative_file,
                        severity,
                        format!(
                            "feature system {:?} is missing required {directory}/ directory",
                            system.domain
                        ),
                        format!("add {directory}/ under {}", system.path.display()),
                    ));
                }
            }
            if !system.path.join(&self.root_index_file).is_file() {
                violations.push(Violation::feature_system_layout(
                    &system.representative_file,
                    severity,
                    format!(
                        "feature system {:?} is missing root {}",
                        system.domain, self.root_index_file
                    ),
                    format!(
                        "add {} under {}",
                        self.root_index_file,
                        system.path.display()
                    ),
                ));
            }
        }

        for file in files {
            if let Some((legacy_root, domain)) = self.legacy_feature(project_root, file) {
                violations.push(Violation::feature_system_layout(
                    file,
                    severity,
                    format!(
                        "legacy feature root {}/{} should use a feature system",
                        display_path_components(&legacy_root),
                        domain
                    ),
                    format!(
                        "move feature code to {}/{}",
                        display_root(&self.systems_roots),
                        domain
                    ),
                ));
            }
            if self.is_feature_component_outside_allowed_roots(project_root, file) {
                violations.push(Violation::feature_system_layout(
                    file,
                    severity,
                    "feature-specific frontend component is outside a feature system".to_string(),
                    format!(
                        "move this component under {}/<domain>/components or an allowed shared UI root",
                        display_root(&self.systems_roots)
                    ),
                ));
            }
            if let Some((domain, expected_file_name, root_level)) =
                self.surface_css_status(project_root, file)
            {
                if !root_level {
                    violations.push(Violation::feature_system_layout(
                        file,
                        severity,
                        format!(
                            "surface CSS for feature system {domain:?} must live at the system root"
                        ),
                        format!(
                            "move this CSS file to {}/{domain}/{expected_file_name}",
                            display_root(&self.systems_roots)
                        ),
                    ));
                } else if file.file_name().and_then(|file_name| file_name.to_str())
                    != Some(expected_file_name.as_str())
                {
                    violations.push(Violation::feature_system_layout(
                        file,
                        severity,
                        format!(
                            "surface CSS for feature system {domain:?} should be named {expected_file_name:?}"
                        ),
                        format!("rename this file to {expected_file_name}"),
                    ));
                }
            }
        }

        violations
    }

    fn discover_systems(
        &self,
        project_root: &Path,
        files: &[PathBuf],
    ) -> BTreeMap<PathBuf, FeatureSystem> {
        let mut systems = BTreeMap::new();
        for file in files {
            let components = project_relative_components(project_root, file);
            let Some((root, domain)) = self.system_root_and_domain(&components) else {
                continue;
            };
            let mut system_components = root.to_vec();
            system_components.push(domain.to_string());
            let system_path = project_root.join(path_from_components(&system_components));
            systems
                .entry(system_path.clone())
                .or_insert_with(|| FeatureSystem {
                    domain: domain.to_string(),
                    path: system_path,
                    representative_file: file.clone(),
                });
        }
        systems
    }

    fn system_root_and_domain<'a>(
        &'a self,
        components: &'a [String],
    ) -> Option<(&'a [String], &'a str)> {
        self.systems_roots.iter().find_map(|root| {
            (components.len() > root.len() && path_has_prefix_components(components, root))
                .then(|| (root.as_slice(), components[root.len()].as_str()))
        })
    }

    fn legacy_feature(&self, project_root: &Path, file: &Path) -> Option<(Vec<String>, String)> {
        let components = project_relative_components(project_root, file);
        self.legacy_roots.iter().find_map(|root| {
            (components.len() > root.len() && path_has_prefix_components(&components, root))
                .then(|| (root.clone(), components[root.len()].clone()))
        })
    }

    fn is_feature_component_outside_allowed_roots(&self, project_root: &Path, file: &Path) -> bool {
        if !is_component_source_file(file) {
            return false;
        }
        let components = project_relative_components(project_root, file);
        if !components
            .iter()
            .any(|component| self.component_directories.contains(component))
        {
            return false;
        }
        if self.system_root_and_domain(&components).is_some()
            || path_under_any_root(&components, &self.allowed_shared_component_roots)
        {
            return false;
        }

        true
    }

    fn surface_css_status(
        &self,
        project_root: &Path,
        file: &Path,
    ) -> Option<(String, String, bool)> {
        if !file
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .is_some_and(|file_name| file_name.ends_with(".css"))
        {
            return None;
        }
        let components = project_relative_components(project_root, file);
        let (root, domain) = self.system_root_and_domain(&components)?;
        let expected_file_name = self.surface_css_name_template.replace("{domain}", domain);
        let root_level = components.len() == root.len() + 2;
        Some((domain.to_string(), expected_file_name, root_level))
    }
}

impl FeatureSystemPublicApiPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let systems_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_PUBLIC_API,
            setting,
            "systemsRoots",
            DEFAULT_SYSTEMS_ROOTS,
        )?;
        let route_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_PUBLIC_API,
            setting,
            "routeRoots",
            DEFAULT_ROUTE_ROOTS,
        )?;
        let allowed_public_entry_points = string_set_option(
            RULE_FEATURE_SYSTEM_PUBLIC_API,
            setting,
            "allowedPublicEntryPoints",
            DEFAULT_FEATURE_SYSTEM_PUBLIC_ENTRY_POINTS,
        )?;
        let reject_wildcard_reexports = bool_option(
            RULE_FEATURE_SYSTEM_PUBLIC_API,
            setting,
            "rejectWildcardReExports",
            true,
        )?;

        Ok(Self {
            systems_roots: path_roots(systems_roots),
            route_roots: path_roots(route_roots),
            allowed_public_entry_points,
            reject_wildcard_reexports,
        })
    }

    fn violations(
        &self,
        project_root: &Path,
        files: &[PathBuf],
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        if self.reject_wildcard_reexports {
            for file in files {
                let Some(location) = self.system_location(project_root, file) else {
                    continue;
                };
                if !self.is_public_entry(&location) {
                    continue;
                }
                let source =
                    fs::read_to_string(file).map_err(|source| OnionCryError::ReadSource {
                        path: file.clone(),
                        source,
                    })?;
                if has_wildcard_reexport(&source) {
                    violations.push(Violation::feature_system_public_api_wildcard(
                        file, severity,
                    ));
                }
            }
        }

        for edge in edges {
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            let Some(target_location) = self.system_location(project_root, target) else {
                continue;
            };
            if self.is_public_entry(&target_location) {
                continue;
            }
            if self
                .system_location(project_root, &edge.source)
                .is_some_and(|source_location| {
                    source_location.system_path == target_location.system_path
                })
            {
                continue;
            }

            violations.push(Violation::feature_system_public_api_internal_import(
                edge,
                target,
                &target_location,
                self.is_route_file(project_root, &edge.source),
                severity,
            ));
        }

        Ok(violations)
    }

    fn system_location(&self, project_root: &Path, file: &Path) -> Option<FeatureSystemLocation> {
        let components = project_relative_components(project_root, file);
        self.systems_roots.iter().find_map(|root| {
            if components.len() <= root.len() || !path_has_prefix_components(&components, root) {
                return None;
            }
            let domain = components[root.len()].clone();
            let system_components = &components[..=root.len()];
            let relative_components = &components[root.len() + 1..];
            Some(FeatureSystemLocation {
                domain,
                system_path: display_path_components(system_components),
                relative_file: display_path_components(relative_components),
            })
        })
    }

    fn is_public_entry(&self, location: &FeatureSystemLocation) -> bool {
        self.allowed_public_entry_points
            .contains(&location.relative_file)
    }

    fn is_route_file(&self, project_root: &Path, file: &Path) -> bool {
        let components = project_relative_components(project_root, file);
        path_under_any_root(&components, &self.route_roots)
    }
}

impl FeatureSystemDependencyFlowPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let systems_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
            setting,
            "systemsRoots",
            DEFAULT_SYSTEMS_ROOTS,
        )?;
        let route_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
            setting,
            "routeRoots",
            DEFAULT_ROUTE_ROOTS,
        )?;
        let allowed_public_entry_points = string_set_option(
            RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
            setting,
            "allowedPublicEntryPoints",
            DEFAULT_FEATURE_SYSTEM_PUBLIC_ENTRY_POINTS,
        )?;
        let adapter_bridge_files = string_set_option(
            RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
            setting,
            "adapterBridgeFiles",
            DEFAULT_FEATURE_SYSTEM_ADAPTER_BRIDGE_FILES,
        )?;
        let allowed_imports = string_set_map_option(
            RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
            setting,
            "allowedImports",
            DEFAULT_FEATURE_SYSTEM_ALLOWED_IMPORTS,
        )?;

        Ok(Self {
            systems_roots: path_roots(systems_roots),
            route_roots: path_roots(route_roots),
            allowed_public_entry_points,
            adapter_bridge_files,
            allowed_imports,
        })
    }

    fn violations(
        &self,
        project_root: &Path,
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        for edge in edges {
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            let source_location = self.system_location(project_root, &edge.source);
            if let Some(source_location) = &source_location {
                let source_area = self.system_area(source_location);
                if self.is_route_file(project_root, target)
                    && !self
                        .is_allowed_area_import(source_area, FeatureSystemDependencyArea::Routes)
                {
                    violations.push(Violation::feature_system_dependency_flow(
                        edge,
                        target,
                        source_area,
                        FeatureSystemDependencyArea::Routes,
                        source_location,
                        severity,
                        "feature systems should be imported by routes, not import route files"
                            .to_string(),
                    ));
                    continue;
                }
            }

            let Some(target_location) = self.system_location(project_root, target) else {
                continue;
            };
            let target_area = self.system_area(&target_location);
            if self.is_public_entry(&target_location) {
                continue;
            }

            match source_location {
                Some(source_location)
                    if source_location.system_path == target_location.system_path =>
                {
                    let source_area = self.system_area(&source_location);
                    if self.is_allowed_same_system_import(
                        source_area,
                        target_area,
                        &source_location,
                    ) {
                        continue;
                    }
                    violations.push(Violation::feature_system_dependency_flow(
                        edge,
                        target,
                        source_area,
                        target_area,
                        &target_location,
                        severity,
                        "route this dependency through an allowed lower area or update allowedImports if the flow is intentional".to_string(),
                    ));
                }
                Some(source_location) => {
                    violations.push(Violation::feature_system_dependency_flow(
                        edge,
                        target,
                        self.system_area(&source_location),
                        target_area,
                        &target_location,
                        severity,
                        format!(
                            "import from the {} public entry point instead",
                            target_location.system_path
                        ),
                    ));
                }
                None => {
                    let source_area = if self.is_route_file(project_root, &edge.source) {
                        FeatureSystemDependencyArea::Routes
                    } else {
                        FeatureSystemDependencyArea::Other
                    };
                    violations.push(Violation::feature_system_dependency_flow(
                        edge,
                        target,
                        source_area,
                        target_area,
                        &target_location,
                        severity,
                        format!(
                            "import from the {} public entry point instead",
                            target_location.system_path
                        ),
                    ));
                }
            }
        }

        violations
    }

    fn system_location(&self, project_root: &Path, file: &Path) -> Option<FeatureSystemLocation> {
        let components = project_relative_components(project_root, file);
        self.systems_roots.iter().find_map(|root| {
            if components.len() <= root.len() || !path_has_prefix_components(&components, root) {
                return None;
            }
            let domain = components[root.len()].clone();
            let system_components = &components[..=root.len()];
            let relative_components = &components[root.len() + 1..];
            Some(FeatureSystemLocation {
                domain,
                system_path: display_path_components(system_components),
                relative_file: display_path_components(relative_components),
            })
        })
    }

    fn system_area(&self, location: &FeatureSystemLocation) -> FeatureSystemDependencyArea {
        if self.is_public_entry(location) {
            return FeatureSystemDependencyArea::PublicEntry;
        }
        FeatureSystemDependencyArea::from_relative_file(&location.relative_file)
    }

    fn is_public_entry(&self, location: &FeatureSystemLocation) -> bool {
        self.allowed_public_entry_points
            .contains(&location.relative_file)
    }

    fn is_route_file(&self, project_root: &Path, file: &Path) -> bool {
        let components = project_relative_components(project_root, file);
        path_under_any_root(&components, &self.route_roots)
    }

    fn is_allowed_same_system_import(
        &self,
        source_area: FeatureSystemDependencyArea,
        target_area: FeatureSystemDependencyArea,
        source_location: &FeatureSystemLocation,
    ) -> bool {
        source_area == target_area
            || self.is_adapter_bridge(source_area, target_area, source_location)
            || self.is_allowed_area_import(source_area, target_area)
    }

    fn is_adapter_bridge(
        &self,
        source_area: FeatureSystemDependencyArea,
        target_area: FeatureSystemDependencyArea,
        source_location: &FeatureSystemLocation,
    ) -> bool {
        source_area == FeatureSystemDependencyArea::Lib
            && target_area == FeatureSystemDependencyArea::Adapters
            && self
                .adapter_bridge_files
                .contains(&source_location.relative_file)
    }

    fn is_allowed_area_import(
        &self,
        source_area: FeatureSystemDependencyArea,
        target_area: FeatureSystemDependencyArea,
    ) -> bool {
        self.allowed_imports
            .get(source_area.config_name())
            .is_some_and(|allowed| allowed.contains(target_area.config_name()))
    }
}

impl FeatureSystemDependencyArea {
    fn from_relative_file(relative_file: &str) -> Self {
        match relative_file.split('/').next().unwrap_or_default() {
            "adapters" => Self::Adapters,
            "lib" => Self::Lib,
            "hooks" => Self::Hooks,
            "contexts" => Self::Contexts,
            "stores" => Self::Stores,
            "components" => Self::Components,
            "guards" => Self::Guards,
            _ => Self::Other,
        }
    }

    fn config_name(self) -> &'static str {
        match self {
            Self::PublicEntry => "public-entry",
            Self::Adapters => "adapters",
            Self::Lib => "lib",
            Self::Hooks => "hooks",
            Self::Contexts => "contexts",
            Self::Stores => "stores",
            Self::Components => "components",
            Self::Guards => "guards",
            Self::Routes => "routes",
            Self::Other => "other",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::PublicEntry => "public entry",
            Self::Adapters => "adapters",
            Self::Lib => "lib",
            Self::Hooks => "hooks",
            Self::Contexts => "contexts",
            Self::Stores => "stores",
            Self::Components => "components",
            Self::Guards => "guards",
            Self::Routes => "routes",
            Self::Other => "outside code",
        }
    }
}

impl FeatureSystemAdapterContractPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let systems_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "systemsRoots",
            DEFAULT_SYSTEMS_ROOTS,
        )?;
        let route_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "routeRoots",
            DEFAULT_ROUTE_ROOTS,
        )?;
        let adapter_directory = string_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "adapterDirectory",
            DEFAULT_FEATURE_SYSTEM_ADAPTER_DIRECTORY,
        )?;
        let adapter_file_name_template = string_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "adapterFileNameTemplate",
            DEFAULT_FEATURE_SYSTEM_ADAPTER_FILE_TEMPLATE,
        )?;
        let api_export_name_template = string_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "apiExportNameTemplate",
            DEFAULT_FEATURE_SYSTEM_API_EXPORT_TEMPLATE,
        )?;
        let error_export_name_template = string_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "errorExportNameTemplate",
            DEFAULT_FEATURE_SYSTEM_API_ERROR_TEMPLATE,
        )?;
        let http_client_names = string_vec_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "httpClientNames",
            DEFAULT_FEATURE_SYSTEM_HTTP_CLIENT_NAMES,
        )?;

        Ok(Self {
            systems_roots: path_roots(systems_roots),
            route_roots: path_roots(route_roots),
            adapter_directory,
            adapter_file_name_template,
            api_export_name_template,
            error_export_name_template,
            http_client_names,
        })
    }

    fn violations(
        &self,
        project_root: &Path,
        files: &[PathBuf],
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let adapter_systems = self.adapter_systems(project_root, files);
        let file_set = files.iter().cloned().collect::<HashSet<_>>();

        for adapter_system in adapter_systems.values() {
            if !file_set.contains(&adapter_system.expected_adapter_file) {
                violations.push(Violation::feature_system_adapter_contract(
                    &adapter_system.representative_file,
                    severity,
                    format!(
                        "feature system adapter layer should expose {}",
                        adapter_system.expected_relative_file
                    ),
                    format!(
                        "add {} as the domain-named adapter file",
                        adapter_system.expected_relative_file
                    ),
                ));
                continue;
            }

            let source =
                fs::read_to_string(&adapter_system.expected_adapter_file).map_err(|source| {
                    OnionCryError::ReadSource {
                        path: adapter_system.expected_adapter_file.clone(),
                        source,
                    }
                })?;
            let expected_api_export =
                render_feature_template(&self.api_export_name_template, &adapter_system.domain);
            if !source_exports_value(&source, &expected_api_export) {
                violations.push(Violation::feature_system_adapter_contract(
                    &adapter_system.expected_adapter_file,
                    severity,
                    format!("adapter file should export namespace object {expected_api_export}"),
                    format!("export a const object named {expected_api_export}"),
                ));
            }

            let expected_error_export =
                render_feature_template(&self.error_export_name_template, &adapter_system.domain);
            if !source_exports_error_class(&source, &expected_error_export) {
                violations.push(Violation::feature_system_adapter_contract(
                    &adapter_system.expected_adapter_file,
                    severity,
                    format!("adapter file should export typed API error {expected_error_export}"),
                    format!("export class {expected_error_export} extends Error"),
                ));
            }

            if source_has_configured_read_call(&source, &self.http_client_names)
                && !source_accepts_and_passes_abort_signal(&source)
            {
                violations.push(Violation::feature_system_adapter_contract(
                    &adapter_system.expected_adapter_file,
                    severity,
                    "adapter read operations should accept and pass an AbortSignal".to_string(),
                    "accept a signal?: AbortSignal parameter and pass it to fetch or the configured HTTP client".to_string(),
                ));
            }
        }

        for edge in edges {
            let Some(source_location) = self.system_location(project_root, &edge.source) else {
                continue;
            };
            if !self.is_adapter_file(&source_location) {
                continue;
            }
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            if self.is_route_file(project_root, target) {
                violations.push(Violation::feature_system_adapter_contract_import(
                    edge,
                    target,
                    FeatureSystemDependencyArea::Routes,
                    severity,
                ));
                continue;
            }
            let Some(target_location) = self.system_location(project_root, target) else {
                continue;
            };
            let target_area =
                FeatureSystemDependencyArea::from_relative_file(&target_location.relative_file);
            if target_area.is_upper_frontend_area() {
                violations.push(Violation::feature_system_adapter_contract_import(
                    edge,
                    target,
                    target_area,
                    severity,
                ));
            }
        }

        Ok(violations)
    }

    fn adapter_systems(
        &self,
        project_root: &Path,
        files: &[PathBuf],
    ) -> BTreeMap<String, FeatureSystemAdapterInfo> {
        let mut systems = BTreeMap::new();

        for file in files {
            let Some(location) = self.system_location(project_root, file) else {
                continue;
            };
            if !self.is_adapter_file(&location) {
                continue;
            }
            let expected_file_name =
                render_feature_template(&self.adapter_file_name_template, &location.domain);
            let expected_relative_file =
                format!("{}/{}", self.adapter_directory, expected_file_name);
            let expected_adapter_file = normalize_path(
                &project_root
                    .join(&location.system_path)
                    .join(&expected_relative_file),
            );
            systems
                .entry(location.system_path.clone())
                .or_insert_with(|| FeatureSystemAdapterInfo {
                    domain: location.domain,
                    representative_file: file.clone(),
                    expected_adapter_file,
                    expected_relative_file,
                });
        }

        systems
    }

    fn system_location(&self, project_root: &Path, file: &Path) -> Option<FeatureSystemLocation> {
        let components = project_relative_components(project_root, file);
        self.systems_roots.iter().find_map(|root| {
            if components.len() <= root.len() || !path_has_prefix_components(&components, root) {
                return None;
            }
            let domain = components[root.len()].clone();
            let system_components = &components[..=root.len()];
            let relative_components = &components[root.len() + 1..];
            Some(FeatureSystemLocation {
                domain,
                system_path: display_path_components(system_components),
                relative_file: display_path_components(relative_components),
            })
        })
    }

    fn is_adapter_file(&self, location: &FeatureSystemLocation) -> bool {
        location
            .relative_file
            .split('/')
            .next()
            .is_some_and(|segment| segment == self.adapter_directory)
    }

    fn is_route_file(&self, project_root: &Path, file: &Path) -> bool {
        let components = project_relative_components(project_root, file);
        path_under_any_root(&components, &self.route_roots)
    }
}

impl FeatureSystemDependencyArea {
    fn is_upper_frontend_area(self) -> bool {
        matches!(
            self,
            Self::Hooks | Self::Contexts | Self::Stores | Self::Components | Self::Guards
        )
    }
}

impl FeatureSystemQueryContractPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let systems_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
            setting,
            "systemsRoots",
            DEFAULT_SYSTEMS_ROOTS,
        )?;
        let route_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
            setting,
            "routeRoots",
            DEFAULT_ROUTE_ROOTS,
        )?;
        let adapter_directory = string_option(
            RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
            setting,
            "adapterDirectory",
            DEFAULT_FEATURE_SYSTEM_ADAPTER_DIRECTORY,
        )?;
        let query_keys_file = string_option(
            RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
            setting,
            "queryKeysFile",
            DEFAULT_FEATURE_SYSTEM_QUERY_KEYS_FILE,
        )?;
        let query_options_file = string_option(
            RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
            setting,
            "queryOptionsFile",
            DEFAULT_FEATURE_SYSTEM_QUERY_OPTIONS_FILE,
        )?;

        Ok(Self {
            systems_roots: path_roots(systems_roots),
            route_roots: path_roots(route_roots),
            adapter_directory,
            query_keys_file,
            query_options_file,
        })
    }

    fn violations(
        &self,
        project_root: &Path,
        files: &[PathBuf],
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Result<Vec<Violation>> {
        let source_by_file = read_source_files(files)?;
        let mut query_states = self.query_states(project_root, files, &source_by_file);
        self.mark_adapter_backed_reads(project_root, edges, &source_by_file, &mut query_states);
        self.mark_route_owned_queries(project_root, edges, &source_by_file, &mut query_states);

        let file_set = files.iter().cloned().collect::<HashSet<_>>();
        let mut violations = Vec::new();

        for state in query_states.values() {
            let query_keys_path =
                self.expected_system_file(project_root, state, &self.query_keys_file);
            if state.requires_query_keys && !file_set.contains(&query_keys_path) {
                violations.push(Violation::feature_system_query_contract(
                    &state.representative_file,
                    severity,
                    format!("{} requires {}", state.domain, self.query_keys_file),
                    format!("add {} under {}", self.query_keys_file, state.system_path),
                ));
            }

            let query_options_path =
                self.expected_system_file(project_root, state, &self.query_options_file);
            if state.requires_query_options && !file_set.contains(&query_options_path) {
                violations.push(Violation::feature_system_query_contract(
                    &state.representative_file,
                    severity,
                    format!("{} requires {}", state.domain, self.query_options_file),
                    format!(
                        "add {} under {}",
                        self.query_options_file, state.system_path
                    ),
                ));
            }

            if let Some(query_options_source) = source_by_file.get(&query_options_path) {
                violations.extend(self.query_options_violations(
                    &query_options_path,
                    query_options_source,
                    project_root,
                    edges,
                    severity,
                ));
            }

            for file in &state.files {
                let Some(source) = source_by_file.get(file) else {
                    continue;
                };
                let Some(location) = self.system_location(project_root, file) else {
                    continue;
                };
                let area = FeatureSystemDependencyArea::from_relative_file(&location.relative_file);
                if area == FeatureSystemDependencyArea::Hooks {
                    violations.extend(self.hook_violations(
                        file,
                        source,
                        project_root,
                        edges,
                        severity,
                    ));
                }
                if area == FeatureSystemDependencyArea::Components
                    && source_declares_query_key(source)
                {
                    violations.push(Violation::feature_system_query_contract(
                        file,
                        severity,
                        "components should not own query keys".to_string(),
                        format!(
                            "move the query key to {} and reuse a query option factory",
                            self.query_keys_file
                        ),
                    ));
                }
            }
        }

        for file in files {
            if !self.is_route_file(project_root, file) {
                continue;
            }
            let Some(source) = source_by_file.get(file) else {
                continue;
            };
            if source_declares_query_key(source) {
                violations.push(Violation::feature_system_query_contract(
                    file,
                    severity,
                    "routes should not own query keys".to_string(),
                    format!(
                        "move the query key to a feature system {} file",
                        self.query_keys_file
                    ),
                ));
            }
        }

        Ok(violations)
    }

    fn query_states(
        &self,
        project_root: &Path,
        files: &[PathBuf],
        source_by_file: &BTreeMap<PathBuf, String>,
    ) -> BTreeMap<String, FeatureSystemQueryState> {
        let mut query_states = BTreeMap::<String, FeatureSystemQueryState>::new();

        for file in files {
            let Some(location) = self.system_location(project_root, file) else {
                continue;
            };
            let source = source_by_file.get(file).map_or("", String::as_str);
            let state = query_states
                .entry(location.system_path.clone())
                .or_insert_with(|| FeatureSystemQueryState {
                    domain: location.domain.clone(),
                    system_path: location.system_path.clone(),
                    representative_file: file.clone(),
                    files: Vec::new(),
                    requires_query_keys: false,
                    requires_query_options: false,
                });
            state.files.push(file.clone());
            if source_has_query_ownership(source) {
                state.requires_query_keys = true;
            }
            if source_uses_query_options_surface(source) {
                state.requires_query_options = true;
            }
        }

        query_states
    }

    fn mark_adapter_backed_reads(
        &self,
        project_root: &Path,
        edges: &[ImportEdge],
        source_by_file: &BTreeMap<PathBuf, String>,
        query_states: &mut BTreeMap<String, FeatureSystemQueryState>,
    ) {
        for edge in edges {
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            let Some(source_location) = self.system_location(project_root, &edge.source) else {
                continue;
            };
            let Some(target_location) = self.system_location(project_root, target) else {
                continue;
            };
            if source_location.system_path != target_location.system_path
                || !self.is_adapter_file(&target_location)
            {
                continue;
            }
            let source = source_by_file.get(&edge.source).map_or("", String::as_str);
            if !source_has_query_ownership(source)
                && source_location.relative_file != self.query_options_file
            {
                continue;
            }
            if let Some(state) = query_states.get_mut(&source_location.system_path) {
                state.requires_query_keys = true;
                state.requires_query_options = true;
            }
        }
    }

    fn mark_route_owned_queries(
        &self,
        project_root: &Path,
        edges: &[ImportEdge],
        source_by_file: &BTreeMap<PathBuf, String>,
        query_states: &mut BTreeMap<String, FeatureSystemQueryState>,
    ) {
        for edge in edges {
            if !self.is_route_file(project_root, &edge.source) {
                continue;
            }
            let source = source_by_file.get(&edge.source).map_or("", String::as_str);
            if !source_uses_query_options_surface(source) {
                continue;
            }
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            let Some(target_location) = self.system_location(project_root, target) else {
                continue;
            };
            if let Some(state) = query_states.get_mut(&target_location.system_path) {
                state.requires_query_keys = true;
                state.requires_query_options = true;
            }
        }
    }

    fn query_options_violations(
        &self,
        file: &Path,
        source: &str,
        project_root: &Path,
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        if !source_imports_and_uses_query_options(source) {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "query option files should import and use queryOptions from @tanstack/react-query"
                    .to_string(),
                "import queryOptions from @tanstack/react-query and wrap option factories with queryOptions".to_string(),
            ));
        }
        if source.contains("queryOptions(")
            && !(source.contains("queryKey") && source.contains("queryFn"))
        {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "query option factories should co-locate queryKey and queryFn".to_string(),
                "define queryKey and queryFn in the same queryOptions factory".to_string(),
            ));
        }
        if self.imports_adapter(project_root, file, edges) && !source_passes_query_signal(source) {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "query functions should pass the query context signal to adapters".to_string(),
                "destructure signal in queryFn and pass it to the adapter call".to_string(),
            ));
        }

        violations
    }

    fn hook_violations(
        &self,
        file: &Path,
        source: &str,
        project_root: &Path,
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        if source_has_query_hook_read(source)
            && (source_declares_query_key(source)
                || source.contains("queryFn")
                || !self.imports_query_options(project_root, file, edges))
        {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "query hooks should reuse factories from lib/query-options.ts".to_string(),
                format!(
                    "import a factory from {} instead of declaring queryKey or queryFn inline",
                    self.query_options_file
                ),
            ));
        }

        if source.contains("useMutation(") && !source_has_mutation_invalidation(source) {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "mutation hooks should invalidate relevant queries in onSuccess or onSettled"
                    .to_string(),
                "add an onSuccess or onSettled handler that calls invalidateQueries".to_string(),
            ));
        }

        if source.contains("onMutate") && !source_has_optimistic_update_contract(source) {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "optimistic cache updates should cancel, snapshot or rollback, and invalidate on settlement".to_string(),
                "include cancelQueries, a previous data snapshot or rollback, and settlement invalidation".to_string(),
            ));
        }

        violations
    }

    fn imports_adapter(&self, project_root: &Path, file: &Path, edges: &[ImportEdge]) -> bool {
        edges.iter().any(|edge| {
            edge.source == file
                && matches!(&edge.resolution, ImportResolution::Local(target) if self
                    .system_location(project_root, target)
                    .is_some_and(|location| self.is_adapter_file(&location)))
        })
    }

    fn imports_query_options(
        &self,
        project_root: &Path,
        file: &Path,
        edges: &[ImportEdge],
    ) -> bool {
        edges.iter().any(|edge| {
            edge.source == file
                && matches!(&edge.resolution, ImportResolution::Local(target) if self
                    .system_location(project_root, target)
                    .is_some_and(|location| location.relative_file == self.query_options_file))
        })
    }

    fn expected_system_file(
        &self,
        project_root: &Path,
        state: &FeatureSystemQueryState,
        relative_file: &str,
    ) -> PathBuf {
        normalize_path(&project_root.join(&state.system_path).join(relative_file))
    }

    fn system_location(&self, project_root: &Path, file: &Path) -> Option<FeatureSystemLocation> {
        let components = project_relative_components(project_root, file);
        self.systems_roots.iter().find_map(|root| {
            if components.len() <= root.len() || !path_has_prefix_components(&components, root) {
                return None;
            }
            let domain = components[root.len()].clone();
            let system_components = &components[..=root.len()];
            let relative_components = &components[root.len() + 1..];
            Some(FeatureSystemLocation {
                domain,
                system_path: display_path_components(system_components),
                relative_file: display_path_components(relative_components),
            })
        })
    }

    fn is_adapter_file(&self, location: &FeatureSystemLocation) -> bool {
        location
            .relative_file
            .split('/')
            .next()
            .is_some_and(|segment| segment == self.adapter_directory)
    }

    fn is_route_file(&self, project_root: &Path, file: &Path) -> bool {
        let components = project_relative_components(project_root, file);
        path_under_any_root(&components, &self.route_roots)
    }
}

impl FeatureImportCounts {
    fn new(source_context: &str) -> Self {
        Self {
            source_context: source_context.to_string(),
            own_context_count: 0,
            other_context_counts: BTreeMap::new(),
        }
    }
}

fn options_object<'a>(
    rule: &str,
    setting: &'a RuleSetting,
) -> Result<Option<&'a Map<String, Value>>> {
    match &setting.options {
        None => Ok(None),
        Some(Value::Object(options)) => Ok(Some(options)),
        Some(_) => Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: "expected rule options object".to_string(),
        }),
    }
}

fn string_vec_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[&str],
) -> Result<Vec<String>> {
    let Some(options) = options_object(rule, setting)? else {
        return Ok(default.iter().map(|value| (*value).to_string()).collect());
    };
    let Some(value) = options.get(key) else {
        return Ok(default.iter().map(|value| (*value).to_string()).collect());
    };

    match value {
        Value::String(value) => Ok(vec![value.clone()]),
        Value::Array(items) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| OnionCryError::InvalidRuleValue {
                        rule: rule.to_string(),
                        message: format!("{key} entries must be strings"),
                    })
            })
            .collect(),
        _ => Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be a string or array of strings"),
        }),
    }
}

fn string_set_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[&str],
) -> Result<HashSet<String>> {
    Ok(string_vec_option(rule, setting, key, default)?
        .into_iter()
        .collect())
}

fn package_pattern_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[&str],
) -> Result<PackageAllowlist> {
    PackageAllowlist::from_patterns(&string_vec_option(rule, setting, key, default)?)
}

fn suffix_map_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[(&str, &[&str])],
) -> Result<BTreeMap<String, Vec<String>>> {
    let Some(options) = options_object(rule, setting)? else {
        return Ok(default_suffix_map(default));
    };
    let Some(value) = options.get(key) else {
        return Ok(default_suffix_map(default));
    };
    let Value::Object(entries) = value else {
        return Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be an object keyed by collection directory"),
        });
    };

    entries
        .iter()
        .map(|(collection, suffixes)| {
            Ok((
                collection.to_string(),
                string_or_array_value(rule, key, suffixes)?,
            ))
        })
        .collect()
}

fn string_set_map_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[(&str, &[&str])],
) -> Result<BTreeMap<String, HashSet<String>>> {
    let mut entries = default
        .iter()
        .map(|(entry_key, values)| {
            (
                (*entry_key).to_string(),
                values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<HashSet<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let Some(options) = options_object(rule, setting)? else {
        return Ok(entries);
    };
    let Some(value) = options.get(key) else {
        return Ok(entries);
    };
    let Value::Object(overrides) = value else {
        return Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be an object keyed by source area"),
        });
    };

    for (entry_key, entry_value) in overrides {
        entries.insert(
            entry_key.to_string(),
            string_or_array_value(rule, key, entry_value)?
                .into_iter()
                .collect(),
        );
    }

    Ok(entries)
}

fn default_suffix_map(default: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    default
        .iter()
        .map(|(collection, suffixes)| {
            (
                (*collection).to_string(),
                suffixes
                    .iter()
                    .map(|suffix| (*suffix).to_string())
                    .collect(),
            )
        })
        .collect()
}

fn string_or_array_value(rule: &str, key: &str, value: &Value) -> Result<Vec<String>> {
    match value {
        Value::String(value) => Ok(vec![value.clone()]),
        Value::Array(items) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| OnionCryError::InvalidRuleValue {
                        rule: rule.to_string(),
                        message: format!("{key} entries must be strings"),
                    })
            })
            .collect(),
        _ => Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} values must be strings or arrays of strings"),
        }),
    }
}

fn usize_option(rule: &str, setting: &RuleSetting, key: &str, default: usize) -> Result<usize> {
    let Some(options) = options_object(rule, setting)? else {
        return Ok(default);
    };
    let Some(value) = options.get(key) else {
        return Ok(default);
    };
    let Some(value) = value.as_u64() else {
        return Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be a positive integer"),
        });
    };
    usize::try_from(value).map_err(|_| OnionCryError::InvalidRuleValue {
        rule: rule.to_string(),
        message: format!("{key} is too large"),
    })
}

fn bool_option(rule: &str, setting: &RuleSetting, key: &str, default: bool) -> Result<bool> {
    let Some(options) = options_object(rule, setting)? else {
        return Ok(default);
    };
    let Some(value) = options.get(key) else {
        return Ok(default);
    };
    value
        .as_bool()
        .ok_or_else(|| OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be true or false"),
        })
}

fn string_option(rule: &str, setting: &RuleSetting, key: &str, default: &str) -> Result<String> {
    let Some(options) = options_object(rule, setting)? else {
        return Ok(default.to_string());
    };
    let Some(value) = options.get(key) else {
        return Ok(default.to_string());
    };
    value
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be a string"),
        })
}

fn render_feature_template(template: &str, domain: &str) -> String {
    template
        .replace("{domain}", domain)
        .replace("{domainCamel}", &domain_camel_case(domain))
        .replace("{DomainPascal}", &domain_pascal_case(domain))
}

fn domain_camel_case(domain: &str) -> String {
    let pascal = domain_pascal_case(domain);
    let mut characters = pascal.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_ascii_lowercase(),
        characters.collect::<String>()
    )
}

fn domain_pascal_case(domain: &str) -> String {
    domain
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            let Some(first) = characters.next() else {
                return String::new();
            };
            format!("{}{}", first.to_ascii_uppercase(), characters.as_str())
        })
        .collect()
}

fn source_exports_value(source: &str, name: &str) -> bool {
    source.contains(&format!("export const {name}"))
        || source.contains(&format!("export let {name}"))
        || source.contains(&format!("export var {name}"))
        || (source.contains(&format!("const {name}"))
            && source.contains(&format!("export {{ {name} }}")))
}

fn source_exports_error_class(source: &str, name: &str) -> bool {
    (source.contains(&format!("export class {name}")) && source.contains("extends Error"))
        || (source.contains(&format!("class {name}"))
            && source.contains("extends Error")
            && source.contains(&format!("export {{ {name} }}")))
}

fn source_has_configured_read_call(source: &str, http_client_names: &[String]) -> bool {
    http_client_names.iter().any(|client| {
        if client == "fetch" {
            source.contains("fetch(")
        } else {
            source.contains(&format!("{client}("))
        }
    })
}

fn source_accepts_and_passes_abort_signal(source: &str) -> bool {
    source.contains("AbortSignal") && source.matches("signal").count() >= 2
}

fn read_source_files(files: &[PathBuf]) -> Result<BTreeMap<PathBuf, String>> {
    let mut sources = BTreeMap::new();
    for file in files {
        if !is_source_file(file) {
            continue;
        }
        let source = fs::read_to_string(file).map_err(|source| OnionCryError::ReadSource {
            path: file.clone(),
            source,
        })?;
        sources.insert(file.clone(), source);
    }
    Ok(sources)
}

fn source_has_query_ownership(source: &str) -> bool {
    source_has_query_hook_read(source)
        || source.contains("prefetchQuery(")
        || source.contains("fetchQuery(")
        || source.contains("ensureQueryData(")
        || source.contains("getQueryData(")
}

fn source_uses_query_options_surface(source: &str) -> bool {
    source_has_query_hook_read(source)
        || source.contains("prefetchQuery(")
        || source.contains("fetchQuery(")
        || source.contains("ensureQueryData(")
        || source.contains("getQueryData(")
}

fn source_has_query_hook_read(source: &str) -> bool {
    source.contains("useQuery(")
        || source.contains("useSuspenseQuery(")
        || source.contains("useInfiniteQuery(")
}

fn source_imports_and_uses_query_options(source: &str) -> bool {
    source.contains("@tanstack/react-query")
        && source.contains("queryOptions")
        && source.contains("queryOptions(")
}

fn source_declares_query_key(source: &str) -> bool {
    source.contains("queryKey")
}

fn source_passes_query_signal(source: &str) -> bool {
    source.contains("queryFn") && source.matches("signal").count() >= 2
}

fn source_has_mutation_invalidation(source: &str) -> bool {
    source.contains("useMutation(")
        && (source.contains("onSuccess") || source.contains("onSettled"))
        && source.contains("invalidateQueries")
}

fn source_has_optimistic_update_contract(source: &str) -> bool {
    source.contains("cancelQueries")
        && (source.contains("previous")
            || source.contains("snapshot")
            || source.contains("rollback"))
        && (source.contains("onSettled") || source.contains("invalidateQueries"))
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

fn git_change_history(project_root: &Path, files: &[PathBuf]) -> Vec<Vec<PathBuf>> {
    let file_set = files
        .iter()
        .map(|file| project_relative_display(project_root, file))
        .collect::<HashSet<_>>();
    let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("log")
        .arg("--name-only")
        .arg("--pretty=format:--onioncry-commit--")
        .arg("--")
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();
    let mut current = BTreeSet::<PathBuf>::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line == "--onioncry-commit--" {
            if current.len() > 1 {
                commits.push(current.into_iter().collect());
            }
            current = BTreeSet::new();
            continue;
        }
        if line.is_empty() || !file_set.contains(line) {
            continue;
        }
        current.insert(normalize_path(&project_root.join(line)));
    }

    if current.len() > 1 {
        commits.push(current.into_iter().collect());
    }

    commits
}

fn shotgun_surgery_findings(
    project_root: &Path,
    files: &[PathBuf],
    history: &[Vec<PathBuf>],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let file_set = files.iter().cloned().collect::<HashSet<_>>();
    let mut change_counts = BTreeMap::<PathBuf, usize>::new();
    let mut cochanges = BTreeMap::<PathBuf, BTreeMap<PathBuf, usize>>::new();

    for commit in history {
        let changed = commit
            .iter()
            .filter(|file| file_set.contains(*file))
            .cloned()
            .collect::<Vec<_>>();
        for file in &changed {
            *change_counts.entry(file.clone()).or_default() += 1;
        }
        for (index, file) in changed.iter().enumerate() {
            for other in changed.iter().skip(index + 1) {
                *cochanges
                    .entry(file.clone())
                    .or_default()
                    .entry(other.clone())
                    .or_default() += 1;
                *cochanges
                    .entry(other.clone())
                    .or_default()
                    .entry(file.clone())
                    .or_default() += 1;
            }
        }
    }

    let mut violations = Vec::new();
    for file in files {
        let severity = rule_policy.effective_severity(RULE_SHOTGUN_SURGERY, project_root, file);
        if severity == Severity::Off {
            continue;
        }
        let rule_setting = rule_policy.effective_rule(RULE_SHOTGUN_SURGERY, project_root, file);
        let min_commit_count =
            usize_option(RULE_SHOTGUN_SURGERY, &rule_setting, "minCommitCount", 4)?;
        let min_related_files =
            usize_option(RULE_SHOTGUN_SURGERY, &rule_setting, "minRelatedFiles", 3)?;
        let min_pair_commit_count =
            usize_option(RULE_SHOTGUN_SURGERY, &rule_setting, "minPairCommitCount", 2)?;
        let commit_count = *change_counts.get(file).unwrap_or(&0);
        if commit_count < min_commit_count {
            continue;
        }
        let related_files = cochanges
            .get(file)
            .map(|related| {
                related
                    .iter()
                    .filter(|(_, count)| **count >= min_pair_commit_count)
                    .count()
            })
            .unwrap_or(0);
        if related_files < min_related_files {
            continue;
        }
        violations.push(Violation::shotgun_surgery(
            file,
            commit_count,
            related_files,
            severity,
        ));
    }

    Ok(violations)
}

impl ContextClassifier {
    fn new(project_root: &Path, context_configs: &BTreeMap<String, ContextConfig>) -> Result<Self> {
        let mut contexts = Vec::new();
        for (name, config) in context_configs {
            contexts.push(CompiledContext {
                name: name.clone(),
                patterns: build_glob_set(&config.patterns)?,
            });
        }
        Ok(Self {
            project_root: project_root.to_path_buf(),
            contexts,
        })
    }

    fn classify(&self, file: &Path) -> ContextClassification<'_> {
        let relative_path = file.strip_prefix(&self.project_root).unwrap_or(file);
        let matched = self
            .contexts
            .iter()
            .filter(|context| context.patterns.is_match(relative_path))
            .collect::<Vec<_>>();

        match matched.as_slice() {
            [] => ContextClassification::Contextless,
            [context] => ContextClassification::Classified(&context.name),
            contexts => ContextClassification::Ambiguous(
                contexts
                    .iter()
                    .map(|context| context.name.clone())
                    .collect(),
            ),
        }
    }
}

impl ContextPolicy {
    fn from(config: &ContextRulesConfig) -> Self {
        Self {
            allow_same_context: config.default.allow_same_context,
            allow_cross_context: config.default.allow_cross_context.iter().cloned().collect(),
        }
    }

    fn is_public_surface(&self, target: &Path, project_root: &Path) -> bool {
        let relative_path = target.strip_prefix(project_root).unwrap_or(target);
        relative_path.components().any(|component| {
            let Component::Normal(segment) = component else {
                return false;
            };
            segment
                .to_str()
                .is_some_and(|segment| self.allow_cross_context.contains(segment))
        })
    }
}

impl LayerClassifier {
    fn new(project_root: &Path, layer_configs: &BTreeMap<String, LayerConfig>) -> Result<Self> {
        let mut layers = Vec::new();
        for (name, config) in layer_configs {
            let patterns = build_glob_set(&config.patterns)?;
            layers.push(CompiledLayer {
                name: name.clone(),
                patterns,
                may_import: config.may_import.iter().cloned().collect(),
            });
        }
        Ok(Self {
            project_root: project_root.to_path_buf(),
            layers,
        })
    }

    fn classify(&self, file: &Path) -> LayerClassification<'_> {
        let relative_path = file.strip_prefix(&self.project_root).unwrap_or(file);
        let matched = self
            .layers
            .iter()
            .filter(|layer| layer.patterns.is_match(relative_path))
            .collect::<Vec<_>>();

        match matched.as_slice() {
            [] => LayerClassification::Unclassified,
            [layer] => LayerClassification::Classified(&layer.name),
            layers => LayerClassification::Ambiguous(
                layers.iter().map(|layer| layer.name.clone()).collect(),
            ),
        }
    }

    fn may_import(&self, from_layer: &str, to_layer: &str) -> bool {
        self.layers
            .iter()
            .find(|layer| layer.name == from_layer)
            .is_some_and(|layer| layer.may_import.contains(to_layer))
    }
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Severity::Off => "off",
            Severity::Warn => "warn",
            Severity::Error => "error",
        }
    }
}

impl ArchitectureMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::CleanArchitecture => "cleanArchitecture",
            Self::VerticalSlice => "verticalSlice",
        }
    }

    fn expected_rule_family(self) -> &'static str {
        match self {
            Self::CleanArchitecture => "cleanarch/*",
            Self::VerticalSlice => "verticalslice/*",
        }
    }
}

impl Default for CleanArchitectureConfig {
    fn default() -> Self {
        Self {
            context_root: default_clean_context_root(),
            layer_path_aliases: default_clean_layer_path_aliases(),
            artifact_folders: default_clean_artifact_folders(),
            artifact_suffixes: default_clean_artifact_suffixes(),
            grouped_artifact_folders: default_clean_grouped_artifact_folders(),
        }
    }
}

impl Default for VerticalSliceConfig {
    fn default() -> Self {
        Self {
            slice_root: default_vertical_slice_root(),
            slice_depth: default_vertical_slice_depth(),
            public_surface: default_vertical_public_surface(),
            artifact_folders: default_vertical_artifact_folders(),
            artifact_suffixes: default_vertical_artifact_suffixes(),
            allowed_global_folders: default_vertical_allowed_global_folders(),
            entry_point_names: default_vertical_entry_point_names(),
            shared_layer_folders: default_vertical_shared_layer_folders(),
        }
    }
}

fn severity_from_str(value: &str) -> Option<Severity> {
    match value {
        "off" => Some(Severity::Off),
        "warn" => Some(Severity::Warn),
        "error" => Some(Severity::Error),
        _ => None,
    }
}

fn parse_rule_map(rules: &Map<String, Value>) -> Result<BTreeMap<String, RuleSetting>> {
    let mut parsed_rules = BTreeMap::new();
    for (rule, value) in rules {
        let canonical_rule = canonical_rule_name(rule)?;
        parsed_rules.insert(canonical_rule.to_string(), parse_rule_setting(rule, value)?);
    }
    Ok(parsed_rules)
}

fn parse_rule_setting(rule: &str, value: &Value) -> Result<RuleSetting> {
    let (severity_text, options) = match value {
        Value::String(severity) => (severity.as_str(), None),
        Value::Array(items) => {
            let severity = items.first().and_then(Value::as_str).ok_or_else(|| {
                OnionCryError::InvalidRuleValue {
                    rule: rule.to_string(),
                    message: "expected [severity, options] with severity off, warn, or error"
                        .to_string(),
                }
            })?;
            if items.len() > 2 {
                return Err(OnionCryError::InvalidRuleValue {
                    rule: rule.to_string(),
                    message: "expected [severity, options] with at most two entries".to_string(),
                });
            }
            (severity, items.get(1).cloned())
        }
        _ => {
            return Err(OnionCryError::InvalidRuleValue {
                rule: rule.to_string(),
                message: "expected severity string or [severity, options]".to_string(),
            });
        }
    };

    let severity =
        severity_from_str(severity_text).ok_or_else(|| OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("invalid severity {severity_text:?}; expected off, warn, or error"),
        })?;

    Ok(RuleSetting { severity, options })
}

fn parse_external_package_layer_policies(
    value: &Value,
    default_severity: Severity,
) -> Result<Vec<ExternalPackageLayerPolicy>> {
    let Value::Array(items) = value else {
        return Err(OnionCryError::InvalidRuleValue {
            rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
            message: "layers must be an array of layer policy objects".to_string(),
        });
    };

    items
        .iter()
        .map(|item| parse_external_package_layer_policy(item, default_severity))
        .collect()
}

fn parse_external_package_layer_policy(
    value: &Value,
    default_severity: Severity,
) -> Result<ExternalPackageLayerPolicy> {
    let Value::Object(entry) = value else {
        return Err(OnionCryError::InvalidRuleValue {
            rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
            message: "layer policy entries must be objects".to_string(),
        });
    };
    let from_layer = entry
        .get("fromLayer")
        .and_then(Value::as_str)
        .ok_or_else(|| OnionCryError::InvalidRuleValue {
            rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
            message: "layer policy entries require fromLayer".to_string(),
        })?;
    let severity = entry
        .get("severity")
        .and_then(Value::as_str)
        .map(|severity| {
            severity_from_str(severity).ok_or_else(|| OnionCryError::InvalidRuleValue {
                rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
                message: format!(
                    "invalid layer severity {severity:?}; expected off, warn, or error"
                ),
            })
        })
        .transpose()?
        .unwrap_or(default_severity);
    let allow = entry
        .get("allow")
        .map(PackageAllowlist::from_value)
        .transpose()?
        .unwrap_or(PackageAllowlist::empty()?);

    Ok(ExternalPackageLayerPolicy {
        from_layer: from_layer.to_string(),
        severity,
        allow,
    })
}

fn canonical_rule_name(rule: &str) -> Result<&'static str> {
    let canonical = match rule {
        RULE_UNCLASSIFIED_FILE | "onion/unclassified-file" => RULE_UNCLASSIFIED_FILE,
        RULE_AMBIGUOUS_LAYER | "onion/ambiguous-layer" => RULE_AMBIGUOUS_LAYER,
        RULE_AMBIGUOUS_CONTEXT | "onion/ambiguous-context" => RULE_AMBIGUOUS_CONTEXT,
        RULE_NO_LAYER_LEAK | "onion/no-layer-leak" => RULE_NO_LAYER_LEAK,
        RULE_NO_FORBIDDEN_IMPORTS | "onion/no-forbidden-imports" => RULE_NO_FORBIDDEN_IMPORTS,
        RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT | "onion/no-cross-context-internal-import" => {
            RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT
        }
        RULE_NO_FRAMEWORK_IN_CORE | "onion/no-framework-in-core" => RULE_NO_FRAMEWORK_IN_CORE,
        RULE_NO_OUTER_DATA_FORMAT_IN_CORE | "onion/no-outer-data-format-in-core" => {
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE
        }
        RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT | "onion/no-public-surface-internal-reexport" => {
            RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT
        }
        RULE_NO_CONTEXT_CYCLE | "onion/no-context-cycle" => RULE_NO_CONTEXT_CYCLE,
        RULE_NO_UNOWNED_SCHEMA_IMPORT | "onion/no-unowned-schema-import" => {
            RULE_NO_UNOWNED_SCHEMA_IMPORT
        }
        RULE_CLEAN_ARTIFACT_PLACEMENT => RULE_CLEAN_ARTIFACT_PLACEMENT,
        RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT => {
            RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT
        }
        RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS => RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS,
        RULE_VERTICAL_SLICE_ENTRY_POINT => RULE_VERTICAL_SLICE_ENTRY_POINT,
        RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS => RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS,
        RULE_NO_CONCRETE_DEPENDENCY | "onion/no-concrete-dependency" => RULE_NO_CONCRETE_DEPENDENCY,
        RULE_FEATURE_ENVY => RULE_FEATURE_ENVY,
        RULE_SHOTGUN_SURGERY => RULE_SHOTGUN_SURGERY,
        RULE_TEST_PLACEMENT => RULE_TEST_PLACEMENT,
        RULE_PATH_NAMING => RULE_PATH_NAMING,
        RULE_FEATURE_SYSTEM_LAYOUT => RULE_FEATURE_SYSTEM_LAYOUT,
        RULE_FEATURE_SYSTEM_PUBLIC_API => RULE_FEATURE_SYSTEM_PUBLIC_API,
        RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW => RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
        RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT => RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
        RULE_FEATURE_SYSTEM_QUERY_CONTRACT => RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
        _ => {
            return Err(OnionCryError::UnknownRule {
                rule: rule.to_string(),
                expected: KNOWN_RULE_NAMES_DISPLAY,
            });
        }
    };

    Ok(canonical)
}

fn validate_architecture_rule_mode(
    mode: ArchitectureMode,
    rules: &BTreeMap<String, RuleSetting>,
) -> Result<()> {
    for (rule, setting) in rules {
        if setting.severity == Severity::Off {
            continue;
        }
        let Some(family) = architecture_rule_family(rule) else {
            continue;
        };
        if family == mode.expected_rule_family() {
            continue;
        }
        return Err(OnionCryError::ArchitectureRuleModeMismatch {
            rule: rule.clone(),
            mode: mode.as_str(),
            expected_family: mode.expected_rule_family(),
        });
    }

    Ok(())
}

fn architecture_rule_family(rule: &str) -> Option<&'static str> {
    if rule.starts_with("cleanarch/") {
        Some("cleanarch/*")
    } else if rule.starts_with("verticalslice/") {
        Some("verticalslice/*")
    } else {
        None
    }
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

fn display_unit_test_directories(unit_test_directories: &HashSet<String>) -> String {
    let mut directories = unit_test_directories.iter().cloned().collect::<Vec<_>>();
    directories.sort();
    directories
        .first()
        .cloned()
        .unwrap_or_else(|| "__tests__".to_string())
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

fn default_rule_setting(rule: &str) -> RuleSetting {
    RuleSetting {
        severity: default_rule_severity(rule),
        options: None,
    }
}

fn default_rule_severity(rule: &str) -> Severity {
    match rule {
        RULE_NO_LAYER_LEAK => Severity::Error,
        RULE_AMBIGUOUS_LAYER => Severity::Error,
        RULE_AMBIGUOUS_CONTEXT => Severity::Error,
        RULE_NO_FORBIDDEN_IMPORTS => Severity::Error,
        RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT => Severity::Error,
        RULE_UNCLASSIFIED_FILE => Severity::Warn,
        RULE_NO_FRAMEWORK_IN_CORE
        | RULE_NO_OUTER_DATA_FORMAT_IN_CORE
        | RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT
        | RULE_NO_CONTEXT_CYCLE
        | RULE_NO_UNOWNED_SCHEMA_IMPORT
        | RULE_CLEAN_ARTIFACT_PLACEMENT
        | RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT
        | RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS
        | RULE_VERTICAL_SLICE_ENTRY_POINT
        | RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS
        | RULE_NO_CONCRETE_DEPENDENCY
        | RULE_FEATURE_ENVY
        | RULE_SHOTGUN_SURGERY
        | RULE_TEST_PLACEMENT
        | RULE_PATH_NAMING
        | RULE_FEATURE_SYSTEM_LAYOUT
        | RULE_FEATURE_SYSTEM_PUBLIC_API
        | RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW
        | RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT
        | RULE_FEATURE_SYSTEM_QUERY_CONTRACT => Severity::Off,
        _ => Severity::Warn,
    }
}

fn default_project_root() -> String {
    ".".to_string()
}

fn default_clean_context_root() -> String {
    DEFAULT_CLEAN_CONTEXT_ROOT.to_string()
}

fn default_clean_layer_path_aliases() -> BTreeMap<String, Vec<String>> {
    default_string_vec_map(DEFAULT_CLEAN_LAYER_ALIASES)
}

fn default_clean_artifact_folders() -> BTreeMap<String, Vec<String>> {
    default_string_vec_map(DEFAULT_CLEAN_ARTIFACT_FOLDERS)
}

fn default_clean_artifact_suffixes() -> BTreeMap<String, Vec<String>> {
    default_string_vec_map(DEFAULT_CLEAN_ARTIFACT_SUFFIXES)
}

fn default_clean_grouped_artifact_folders() -> Vec<String> {
    DEFAULT_CLEAN_GROUPED_ARTIFACT_FOLDERS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

fn default_vertical_slice_root() -> String {
    DEFAULT_VERTICAL_SLICE_ROOT.to_string()
}

fn default_vertical_slice_depth() -> usize {
    DEFAULT_VERTICAL_SLICE_DEPTH
}

fn default_vertical_public_surface() -> Vec<String> {
    DEFAULT_VERTICAL_PUBLIC_SURFACE
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

fn default_vertical_artifact_folders() -> Vec<String> {
    DEFAULT_VERTICAL_ARTIFACT_FOLDERS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

fn default_vertical_artifact_suffixes() -> BTreeMap<String, Vec<String>> {
    default_string_vec_map(DEFAULT_VERTICAL_ARTIFACT_SUFFIXES)
}

fn default_vertical_allowed_global_folders() -> Vec<String> {
    DEFAULT_VERTICAL_ALLOWED_GLOBAL_FOLDERS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

fn default_vertical_entry_point_names() -> Vec<String> {
    DEFAULT_VERTICAL_ENTRY_POINT_NAMES
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

fn default_vertical_shared_layer_folders() -> Vec<String> {
    DEFAULT_VERTICAL_SHARED_LAYER_FOLDERS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

fn default_string_vec_map(default: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    default
        .iter()
        .map(|(key, values)| {
            (
                (*key).to_string(),
                values.iter().map(|value| (*value).to_string()).collect(),
            )
        })
        .collect()
}

impl Default for ContextRuleDefaultConfig {
    fn default() -> Self {
        Self {
            allow_same_context: default_allow_same_context(),
            allow_cross_context: Vec::new(),
        }
    }
}

fn default_allow_same_context() -> bool {
    true
}

fn default_include_patterns() -> Vec<String> {
    vec![
        "**/*.js".to_string(),
        "**/*.jsx".to_string(),
        "**/*.ts".to_string(),
        "**/*.tsx".to_string(),
    ]
}

fn scan_imports(path: &Path, source: &str) -> Result<Vec<RawImport>> {
    let source_type = SourceType::from_path(path).map_err(|source| OnionCryError::ParseSource {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        let message = parsed
            .errors
            .first()
            .map_or_else(|| "parser panicked".to_string(), ToString::to_string);
        return Err(OnionCryError::ParseSource {
            path: path.to_path_buf(),
            message,
        });
    }

    let mut imports = Vec::new();
    for (specifier, requested_modules) in &parsed.module_record.requested_modules {
        for requested_module in requested_modules {
            imports.push(RawImport {
                specifier: specifier.to_string(),
                kind: if requested_module.is_import {
                    ImportKind::StaticImport
                } else {
                    ImportKind::ReExport
                },
                type_only: requested_module.is_type,
                span: requested_module.span,
            });
        }
    }

    let mut visitor = RuntimeImportVisitor::default();
    visitor.visit_program(&parsed.program);
    imports.extend(visitor.imports);
    imports.sort_by_key(|import| import.span.start);
    Ok(imports)
}

#[derive(Default)]
struct RuntimeImportVisitor {
    imports: Vec<RawImport>,
}

impl<'a> Visit<'a> for RuntimeImportVisitor {
    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if let Expression::StringLiteral(source) = &import.source {
            self.imports.push(RawImport {
                specifier: source.value.to_string(),
                kind: ImportKind::DynamicImport,
                type_only: false,
                span: source.span,
            });
        }
        walk::walk_import_expression(self, import);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(specifier) = string_literal_require_specifier(call) {
            self.imports.push(RawImport {
                specifier: specifier.value.to_string(),
                kind: ImportKind::Require,
                type_only: false,
                span: specifier.span,
            });
        }
        walk::walk_call_expression(self, call);
    }
}

fn string_literal_require_specifier<'a>(
    call: &'a CallExpression<'a>,
) -> Option<&'a oxc_ast::ast::StringLiteral<'a>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if callee.name != "require" || call.arguments.len() != 1 {
        return None;
    }
    let Argument::StringLiteral(specifier) = &call.arguments[0] else {
        return None;
    };
    Some(specifier)
}

fn resolve_import(
    specifier: &str,
    source_file: &Path,
    project_root: &Path,
    aliases: &[AliasMapping],
) -> ImportResolution {
    if specifier.starts_with("./") || specifier.starts_with("../") {
        let Some(source_dir) = source_file.parent() else {
            return ImportResolution::UnresolvedLocal;
        };
        return resolve_local_candidate(&source_dir.join(specifier))
            .map_or(ImportResolution::UnresolvedLocal, ImportResolution::Local);
    }

    for alias in aliases {
        if let Some(remainder) = specifier.strip_prefix(&alias.prefix) {
            let remainder = remainder.strip_prefix('/').unwrap_or(remainder);
            let candidate = project_root.join(&alias.target).join(remainder);
            return resolve_local_candidate(&candidate)
                .map_or(ImportResolution::UnresolvedLocal, ImportResolution::Local);
        }
    }

    ImportResolution::External
}

fn resolve_local_candidate(candidate: &Path) -> Option<PathBuf> {
    if candidate.is_file() {
        return Some(normalize_path(candidate));
    }

    for extension in SOURCE_EXTENSIONS {
        let with_extension = append_extension(candidate, extension);
        if with_extension.is_file() {
            return Some(normalize_path(&with_extension));
        }
    }

    for extension in SOURCE_EXTENSIONS {
        let index_path = candidate.join(format!("index.{extension}"));
        if index_path.is_file() {
            return Some(normalize_path(&index_path));
        }
    }

    None
}

fn append_extension(path: &Path, extension: &str) -> PathBuf {
    let mut path_with_extension = path.as_os_str().to_os_string();
    path_with_extension.push(".");
    path_with_extension.push(extension);
    PathBuf::from(path_with_extension)
}

fn line_column(source: &str, byte_index: usize) -> (usize, usize) {
    let safe_index = byte_index.min(source.len());
    let prefix = &source[..safe_index];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix, |(_, current_line)| current_line)
        .chars()
        .count()
        + 1;
    (line, column)
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
