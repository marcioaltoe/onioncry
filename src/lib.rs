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
use thiserror::Error;
use walkdir::WalkDir;

pub const DEFAULT_CONFIG_FILE: &str = ".onioncryrc.jsonc";
const RULE_UNRESOLVED_IMPORT: &str = "onion/unresolved-import";
const RULE_UNCLASSIFIED_FILE: &str = "onion/unclassified-file";
const RULE_AMBIGUOUS_LAYER: &str = "onion/ambiguous-layer";
const RULE_AMBIGUOUS_CONTEXT: &str = "onion/ambiguous-context";
const RULE_NO_LAYER_LEAK: &str = "onion/no-layer-leak";
const RULE_NO_FORBIDDEN_IMPORTS: &str = "onion/no-forbidden-imports";
const RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT: &str = "onion/no-cross-context-internal-import";
const RULE_CIRCULAR_DEPENDENCY: &str = "onion/circular-dependency";
const KNOWN_RULE_NAMES: &[&str] = &[
    RULE_UNRESOLVED_IMPORT,
    RULE_UNCLASSIFIED_FILE,
    RULE_AMBIGUOUS_LAYER,
    RULE_AMBIGUOUS_CONTEXT,
    RULE_NO_LAYER_LEAK,
    RULE_NO_FORBIDDEN_IMPORTS,
    RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT,
    RULE_CIRCULAR_DEPENDENCY,
];
const KNOWN_RULE_NAMES_DISPLAY: &str = "onion/unresolved-import, onion/unclassified-file, onion/ambiguous-layer, onion/ambiguous-context, onion/no-layer-leak, onion/no-forbidden-imports, onion/no-cross-context-internal-import, onion/circular-dependency";
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
    "onion/no-layer-leak": "error",
    "onion/no-cross-context-internal-import": "error",
    "onion/no-forbidden-imports": ["error", {
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
    "onion/unresolved-import": "warn",
    "onion/circular-dependency": "warn",
    "onion/unclassified-file": "warn"
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
    let mut violations = collect_unresolved_import_violations(edges, project_root, rule_policy);
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
    violations.extend(collect_circular_dependency_violations(
        project_root,
        files,
        edges,
        rule_policy,
    ));
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

fn collect_unresolved_import_violations(
    edges: &[ImportEdge],
    project_root: &Path,
    rule_policy: &RulePolicy,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    for edge in edges {
        if !matches!(edge.resolution, ImportResolution::UnresolvedLocal) {
            continue;
        }
        let severity =
            rule_policy.effective_severity(RULE_UNRESOLVED_IMPORT, project_root, &edge.source);
        if severity == Severity::Off {
            continue;
        }
        violations.push(Violation::unresolved_import(edge, severity));
    }
    violations
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

fn collect_circular_dependency_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Vec<Violation> {
    let file_set = files.iter().cloned().collect::<HashSet<_>>();
    let graph = build_local_dependency_graph(edges, &file_set);
    let cycles = find_canonical_cycles(&graph);
    let mut violations = Vec::new();

    for cycle in cycles {
        let Some(source) = cycle.first() else {
            continue;
        };
        let severity =
            rule_policy.effective_severity(RULE_CIRCULAR_DEPENDENCY, project_root, source);
        if severity == Severity::Off {
            continue;
        }
        violations.push(Violation::circular_dependency(
            source,
            &cycle,
            project_root,
            severity,
        ));
    }

    violations
}

fn build_local_dependency_graph(
    edges: &[ImportEdge],
    file_set: &HashSet<PathBuf>,
) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut graph = BTreeMap::<PathBuf, BTreeSet<PathBuf>>::new();

    for edge in edges {
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        if !file_set.contains(target) {
            continue;
        }
        graph
            .entry(edge.source.clone())
            .or_default()
            .insert(target.clone());
        graph.entry(target.clone()).or_default();
    }

    graph
        .into_iter()
        .map(|(source, targets)| (source, targets.into_iter().collect()))
        .collect()
}

fn find_canonical_cycles(graph: &BTreeMap<PathBuf, Vec<PathBuf>>) -> Vec<Vec<PathBuf>> {
    let mut cycles = Vec::new();
    let nodes = graph.keys().cloned().collect::<Vec<_>>();

    for start in nodes {
        let mut stack = vec![start.clone()];
        let mut in_stack = HashSet::from([start.clone()]);
        find_cycles_from(
            &start,
            &start,
            graph,
            &mut stack,
            &mut in_stack,
            &mut cycles,
        );
    }

    cycles
}

fn find_cycles_from(
    start: &PathBuf,
    current: &PathBuf,
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
    stack: &mut Vec<PathBuf>,
    in_stack: &mut HashSet<PathBuf>,
    cycles: &mut Vec<Vec<PathBuf>>,
) {
    let Some(targets) = graph.get(current) else {
        return;
    };

    for target in targets {
        if target == start && stack.len() > 1 {
            let mut cycle = stack.clone();
            cycle.push(start.clone());
            cycles.push(cycle);
            continue;
        }
        if target < start || in_stack.contains(target) {
            continue;
        }

        stack.push(target.clone());
        in_stack.insert(target.clone());
        find_cycles_from(start, target, graph, stack, in_stack, cycles);
        in_stack.remove(target);
        stack.pop();
    }
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

pub fn render_pretty(report: &CheckReport) -> String {
    let mut output = format!(
        "status: {}\nfiles: {}\nwarnings: {}\nerrors: {}\nviolations: {}\n",
        report.status.as_str(),
        report.summary.file_count,
        report.summary.warning_count,
        report.summary.error_count,
        report.summary.violation_count
    );
    for violation in &report.violations {
        output.push_str(&format!(
            "{} {} {}",
            violation.severity, violation.rule, violation.file
        ));
        if let Some(import_specifier) = &violation.import_specifier {
            output.push_str(&format!(" import {import_specifier}"));
        }
        if let Some(package_name) = &violation.package_name {
            output.push_str(&format!(" package {package_name}"));
        }
        if let (Some(from_layer), Some(to_layer)) = (&violation.from_layer, &violation.to_layer) {
            output.push_str(&format!(" {from_layer}->{to_layer}"));
        }
        if let (Some(from_context), Some(to_context)) =
            (&violation.from_context, &violation.to_context)
        {
            output.push_str(&format!(" {from_context}->{to_context}"));
        }
        output.push_str(&format!(": {}\n", violation.message));
        if let Some(suggestion) = &violation.suggestion {
            output.push_str(&format!("suggestion: {suggestion}\n"));
        }
    }
    output
}

pub fn render_explain_pretty(report: &ExplainReport) -> String {
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
        output.push_str(&format!(
            "- {} {}: {}\n",
            violation.severity, violation.rule, violation.message
        ));
        if let Some(suggestion) = &violation.suggestion {
            output.push_str(&format!("  suggestion: {suggestion}\n"));
        }
    }

    output
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
        aliases.sort_by(|left, right| right.prefix.len().cmp(&left.prefix.len()));
        aliases
    }
}

impl Violation {
    fn unresolved_import(edge: &ImportEdge, severity: Severity) -> Self {
        Self {
            rule: RULE_UNRESOLVED_IMPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!("could not resolve local import {}", edge.specifier),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: None,
            matched_layers: None,
            matched_contexts: None,
        }
    }

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
                "add {package_name:?} to the onion/no-forbidden-imports allowlist for {from_layer} only if this package is domain-safe for that layer"
            )),
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

    fn circular_dependency(
        source: &Path,
        cycle: &[PathBuf],
        project_root: &Path,
        severity: Severity,
    ) -> Self {
        let cycle_path = cycle
            .iter()
            .map(|path| project_relative_display(project_root, path))
            .collect::<Vec<_>>();
        Self {
            rule: RULE_CIRCULAR_DEPENDENCY.to_string(),
            severity: severity.as_str().to_string(),
            message: format!("circular dependency: {}", cycle_path.join(" -> ")),
            file: source.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: Some(cycle_path),
            suggestion: Some(
                "break the cycle by moving shared contracts or reversing one dependency"
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
        validate_rule_name(rule)?;
        parsed_rules.insert(rule.clone(), parse_rule_setting(rule, value)?);
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

fn validate_rule_name(rule: &str) -> Result<()> {
    if KNOWN_RULE_NAMES.contains(&rule) {
        Ok(())
    } else {
        Err(OnionCryError::UnknownRule {
            rule: rule.to_string(),
            expected: KNOWN_RULE_NAMES_DISPLAY,
        })
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
        RULE_UNRESOLVED_IMPORT | RULE_UNCLASSIFIED_FILE | RULE_CIRCULAR_DEPENDENCY => {
            Severity::Warn
        }
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

    if candidate.extension().is_none() {
        for extension in SOURCE_EXTENSIONS {
            let with_extension = candidate.with_extension(extension);
            if with_extension.is_file() {
                return Some(normalize_path(&with_extension));
            }
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
