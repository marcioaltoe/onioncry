use crate::{OnionCryError, Result, normalize_path, resolve_against};
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub version: Value,
    pub project: ProjectConfig,
    #[serde(default)]
    pub architecture: ArchitectureConfig,
    #[serde(default)]
    #[schemars(with = "BTreeMap<String, Value>")]
    pub aliases: Map<String, Value>,
    #[serde(default)]
    pub layers: BTreeMap<String, LayerConfig>,
    #[serde(default)]
    pub contexts: BTreeMap<String, ContextConfig>,
    #[serde(default)]
    pub context_rules: ContextRulesConfig,
    #[serde(default)]
    #[schemars(schema_with = "rule_config_map_schema")]
    pub rules: Map<String, Value>,
    #[serde(default)]
    pub overrides: Vec<OverrideConfig>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchitectureConfig {
    #[serde(default)]
    pub mode: ArchitectureMode,
    #[serde(default)]
    pub clean_architecture: CleanArchitectureConfig,
    #[serde(default)]
    pub vertical_slice: VerticalSliceConfig,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ArchitectureMode {
    #[default]
    CleanArchitecture,
    VerticalSlice,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CleanArchitectureConfig {
    #[serde(default = "super::defaults::default_clean_context_root")]
    pub context_root: String,
    #[serde(default = "super::defaults::default_clean_layer_path_aliases")]
    pub layer_path_aliases: BTreeMap<String, Vec<String>>,
    #[serde(default = "super::defaults::default_clean_artifact_folders")]
    pub artifact_folders: BTreeMap<String, Vec<String>>,
    #[serde(default = "super::defaults::default_clean_artifact_suffixes")]
    pub artifact_suffixes: BTreeMap<String, Vec<String>>,
    #[serde(default = "super::defaults::default_clean_grouped_artifact_folders")]
    pub grouped_artifact_folders: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerticalSliceConfig {
    #[serde(default = "super::defaults::default_vertical_slice_root")]
    pub slice_root: String,
    #[serde(default = "super::defaults::default_vertical_slice_depth")]
    pub slice_depth: usize,
    #[serde(default = "super::defaults::default_vertical_public_surface")]
    pub public_surface: Vec<String>,
    #[serde(default = "super::defaults::default_vertical_artifact_folders")]
    pub artifact_folders: Vec<String>,
    #[serde(default = "super::defaults::default_vertical_artifact_suffixes")]
    pub artifact_suffixes: BTreeMap<String, Vec<String>>,
    #[serde(default = "super::defaults::default_vertical_allowed_global_folders")]
    pub allowed_global_folders: Vec<String>,
    #[serde(default = "super::defaults::default_vertical_entry_point_names")]
    pub entry_point_names: Vec<String>,
    #[serde(default = "super::defaults::default_vertical_shared_layer_folders")]
    pub shared_layer_folders: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    #[serde(default = "super::defaults::default_project_root")]
    pub root: String,
    #[serde(default = "super::defaults::default_include_patterns")]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LayerConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub may_import: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextRulesConfig {
    #[serde(default)]
    pub default: ContextRuleDefaultConfig,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextRuleDefaultConfig {
    #[serde(default = "super::defaults::default_allow_same_context")]
    pub allow_same_context: bool,
    #[serde(default)]
    pub allow_cross_context: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OverrideConfig {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    #[schemars(schema_with = "rule_config_map_schema")]
    pub rules: Map<String, Value>,
}

#[derive(Debug)]
pub struct LoadedConfig {
    pub path: PathBuf,
    pub config_dir: PathBuf,
    pub config: Config,
}

#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq)]
#[schemars(rename_all = "lowercase")]
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

fn rule_config_map_schema(generator: &mut SchemaGenerator) -> Schema {
    let severity_schema = generator.subschema_for::<Severity>();
    let tuple_schema = json_schema!({
        "type": "array",
        "minItems": 1,
        "maxItems": 2,
        "prefixItems": [
            severity_schema,
            true
        ]
    });
    let rule_value_schema = json_schema!({
        "anyOf": [
            severity_schema,
            tuple_schema
        ]
    });

    json_schema!({
        "type": "object",
        "additionalProperties": rule_value_schema,
        "default": {}
    })
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
