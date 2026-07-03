use crate::*;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
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
pub(crate) const INIT_CONFIG_TEMPLATE: &str = r#"{
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
}

pub(crate) fn options_object<'a>(
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

pub(crate) fn string_vec_option(
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

pub(crate) fn string_set_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[&str],
) -> Result<HashSet<String>> {
    Ok(string_vec_option(rule, setting, key, default)?
        .into_iter()
        .collect())
}

pub(crate) fn package_pattern_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[&str],
) -> Result<PackageAllowlist> {
    PackageAllowlist::from_patterns(&string_vec_option(rule, setting, key, default)?)
}

pub(crate) fn suffix_map_option(
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

pub(crate) fn string_set_map_option(
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

pub(crate) fn usize_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: usize,
) -> Result<usize> {
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

pub(crate) fn bool_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: bool,
) -> Result<bool> {
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

pub(crate) fn string_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &str,
) -> Result<String> {
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

impl Severity {
    pub(crate) fn as_str(self) -> &'static str {
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

pub(crate) fn parse_rule_map(rules: &Map<String, Value>) -> Result<BTreeMap<String, RuleSetting>> {
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

pub(crate) fn parse_external_package_layer_policies(
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

pub(crate) fn validate_architecture_rule_mode(
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

pub(crate) fn default_rule_setting(rule: &str) -> RuleSetting {
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
