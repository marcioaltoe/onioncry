use super::types::{CleanArchitectureConfig, ContextRuleDefaultConfig, VerticalSliceConfig};
use std::collections::BTreeMap;

pub(crate) const DEFAULT_TEST_FILE_SUFFIXES: &[&str] = &[
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
pub(crate) const DEFAULT_COLLECTION_DIRECTORIES: &[&str] = &[
    "entities",
    "repositories",
    "value-objects",
    "use-cases",
    "events",
    "services",
    "gateways",
    "dtos",
];
pub(crate) const DEFAULT_LAYER_DIRECTORIES: &[&str] = &["domain", "application", "infra", "shared"];
pub(crate) const DEFAULT_FEATURE_ROOTS: &[&str] = &["src"];
pub(crate) const DEFAULT_IGNORED_PATH_DIRECTORIES: &[&str] = &["__tests__"];
pub(crate) const DEFAULT_SUFFIXES_BY_COLLECTION: &[(&str, &[&str])] = &[
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
pub(crate) const DEFAULT_SYSTEMS_ROOTS: &[&str] = &["packages/frontend/src/systems"];
pub(crate) const DEFAULT_FEATURE_SYSTEM_REQUIRED_DIRECTORIES: &[&str] = &["components", "lib"];
pub(crate) const DEFAULT_FEATURE_SYSTEM_OPTIONAL_DIRECTORIES: &[&str] =
    &["hooks", "adapters", "contexts", "stores", "guards"];
pub(crate) const DEFAULT_ALLOWED_SHARED_COMPONENT_ROOTS: &[&str] =
    &["packages/frontend/src/components/ui"];
pub(crate) const DEFAULT_LEGACY_FEATURE_ROOTS: &[&str] = &["packages/frontend/src/features"];
pub(crate) const DEFAULT_COMPONENT_DIRECTORIES: &[&str] = &["components"];
pub(crate) const DEFAULT_FEATURE_SYSTEM_ROOT_INDEX_FILE: &str = "index.ts";
pub(crate) const DEFAULT_SURFACE_CSS_NAME_TEMPLATE: &str = "{domain}.css";
pub(crate) const DEFAULT_ROUTE_ROOTS: &[&str] =
    &["packages/frontend/src/routes", "packages/frontend/src/app"];
pub(crate) const DEFAULT_FEATURE_SYSTEM_PUBLIC_ENTRY_POINTS: &[&str] = &["index.ts"];
pub(crate) const DEFAULT_FEATURE_SYSTEM_ADAPTER_BRIDGE_FILES: &[&str] = &["lib/query-options.ts"];
pub(crate) const DEFAULT_FEATURE_SYSTEM_ADAPTER_DIRECTORY: &str = "adapters";
pub(crate) const DEFAULT_FEATURE_SYSTEM_ADAPTER_FILE_TEMPLATE: &str = "{domain}-api.ts";
pub(crate) const DEFAULT_FEATURE_SYSTEM_API_EXPORT_TEMPLATE: &str = "{domainCamel}Api";
pub(crate) const DEFAULT_FEATURE_SYSTEM_API_ERROR_TEMPLATE: &str = "{DomainPascal}ApiError";
pub(crate) const DEFAULT_FEATURE_SYSTEM_HTTP_CLIENT_NAMES: &[&str] =
    &["fetch", "http.get", "apiClient.get", "client.get"];
pub(crate) const DEFAULT_FEATURE_SYSTEM_QUERY_KEYS_FILE: &str = "lib/query-keys.ts";
pub(crate) const DEFAULT_FEATURE_SYSTEM_QUERY_OPTIONS_FILE: &str = "lib/query-options.ts";
pub(crate) const DEFAULT_FEATURE_SYSTEM_ALLOWED_IMPORTS: &[(&str, &[&str])] = &[
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

pub(super) fn default_project_root() -> String {
    ".".to_string()
}

pub(super) fn default_clean_context_root() -> String {
    DEFAULT_CLEAN_CONTEXT_ROOT.to_string()
}

pub(super) fn default_clean_layer_path_aliases() -> BTreeMap<String, Vec<String>> {
    default_string_vec_map(DEFAULT_CLEAN_LAYER_ALIASES)
}

pub(super) fn default_clean_artifact_folders() -> BTreeMap<String, Vec<String>> {
    default_string_vec_map(DEFAULT_CLEAN_ARTIFACT_FOLDERS)
}

pub(super) fn default_clean_artifact_suffixes() -> BTreeMap<String, Vec<String>> {
    default_string_vec_map(DEFAULT_CLEAN_ARTIFACT_SUFFIXES)
}

pub(super) fn default_clean_grouped_artifact_folders() -> Vec<String> {
    DEFAULT_CLEAN_GROUPED_ARTIFACT_FOLDERS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub(super) fn default_vertical_slice_root() -> String {
    DEFAULT_VERTICAL_SLICE_ROOT.to_string()
}

pub(super) fn default_vertical_slice_depth() -> usize {
    DEFAULT_VERTICAL_SLICE_DEPTH
}

pub(super) fn default_vertical_public_surface() -> Vec<String> {
    DEFAULT_VERTICAL_PUBLIC_SURFACE
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub(super) fn default_vertical_artifact_folders() -> Vec<String> {
    DEFAULT_VERTICAL_ARTIFACT_FOLDERS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub(super) fn default_vertical_artifact_suffixes() -> BTreeMap<String, Vec<String>> {
    default_string_vec_map(DEFAULT_VERTICAL_ARTIFACT_SUFFIXES)
}

pub(super) fn default_vertical_allowed_global_folders() -> Vec<String> {
    DEFAULT_VERTICAL_ALLOWED_GLOBAL_FOLDERS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub(super) fn default_vertical_entry_point_names() -> Vec<String> {
    DEFAULT_VERTICAL_ENTRY_POINT_NAMES
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub(super) fn default_vertical_shared_layer_folders() -> Vec<String> {
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

pub(super) fn default_allow_same_context() -> bool {
    true
}

pub(super) fn default_include_patterns() -> Vec<String> {
    vec![
        "**/*.js".to_string(),
        "**/*.jsx".to_string(),
        "**/*.ts".to_string(),
        "**/*.tsx".to_string(),
    ]
}
