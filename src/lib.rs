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
const RULE_NO_CONCRETE_DEPENDENCY: &str = "solid/no-concrete-dependency";
const RULE_FEATURE_ENVY: &str = "codesmells/feature-envy";
const RULE_SHOTGUN_SURGERY: &str = "codesmells/shotgun-surgery";
const KNOWN_RULE_NAMES_DISPLAY: &str = "cleanarch/no-layer-leak, cleanarch/no-forbidden-imports, cleanarch/no-cross-context-internal-import, cleanarch/no-framework-in-core, cleanarch/no-outer-data-format-in-core, cleanarch/no-public-surface-internal-reexport, cleanarch/no-context-cycle, cleanarch/no-unowned-schema-import, cleanarch/unclassified-file, cleanarch/ambiguous-layer, cleanarch/ambiguous-context, solid/no-concrete-dependency, codesmells/feature-envy, codesmells/shotgun-surgery";
const INIT_CONFIG_TEMPLATE: &str = r#"{
  "$schema": "./onioncry.schema.json",
  "version": 1,
  "project": {
    "root": ".",
    // TODO: adjust the file universe for your source layout.
    "include": ["src/**/*.{ts,tsx,js,jsx,mts,cts,mjs,cjs}"],
    "exclude": ["node_modules/**", "dist/**", "build/**", "coverage/**"]
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
    "solid/no-concrete-dependency": "warn",
    "codesmells/feature-envy": "warn",
    "codesmells/shotgun-surgery": "off",
    "cleanarch/unclassified-file": "warn"
  },
  // TODO: use overrides for temporary policy exceptions, not file selection.
  "overrides": []
}
"#;

#[derive(Debug, Error)]
pub enum OnionCryError {
    #[error("could not find {DEFAULT_CONFIG_FILE}; pass --config <path> to use a different file")]
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
}

pub type Result<T> = std::result::Result<T, OnionCryError>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub version: Value,
    pub project: ProjectConfig,
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
            let path = cwd.join(DEFAULT_CONFIG_FILE);
            if path.exists() {
                Ok(path)
            } else {
                Err(OnionCryError::MissingDefaultConfig)
            }
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
        "onioncry-llm-report v1\nstatus: {}\nfilesChecked: {}\nproblemCount: {}\nerrorCount: {}\nwarningCount: {}\ngroupCount: {}\n",
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
        RULE_NO_CONCRETE_DEPENDENCY => {
            "Core layers should depend on abstractions rather than concrete details."
        }
        RULE_FEATURE_ENVY => {
            "A file that mostly imports another context may contain behavior owned by that context."
        }
        RULE_SHOTGUN_SURGERY => {
            "Files that repeatedly change with many companions may hide scattered responsibilities."
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
}

impl RulePolicy {
    fn new(config: &Config) -> Result<Self> {
        let base_rules = parse_rule_map(&config.rules)?;
        let mut overrides = Vec::new();

        for override_config in &config.overrides {
            overrides.push(CompiledOverride {
                files: build_glob_set(&override_config.files)?,
                rules: parse_rule_map(&override_config.rules)?,
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
        RULE_NO_CONCRETE_DEPENDENCY | "onion/no-concrete-dependency" => RULE_NO_CONCRETE_DEPENDENCY,
        RULE_FEATURE_ENVY => RULE_FEATURE_ENVY,
        RULE_SHOTGUN_SURGERY => RULE_SHOTGUN_SURGERY,
        _ => {
            return Err(OnionCryError::UnknownRule {
                rule: rule.to_string(),
                expected: KNOWN_RULE_NAMES_DISPLAY,
            });
        }
    };

    Ok(canonical)
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
        | RULE_NO_CONCRETE_DEPENDENCY
        | RULE_FEATURE_ENVY
        | RULE_SHOTGUN_SURGERY => Severity::Off,
        _ => Severity::Warn,
    }
}

fn default_project_root() -> String {
    ".".to_string()
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
