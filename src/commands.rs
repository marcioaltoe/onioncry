use crate::rules::catalog::RULE_NO_FORBIDDEN_IMPORTS;
use crate::rules::{RuleCollectionContext, collect_rule_violations};
use crate::{
    BoundaryExplanation, CheckReport, Config, ContextConfig, ContextPolicy, DEFAULT_BASELINE_FILE,
    DEFAULT_CONFIG_FILE, ExplainReport, ExternalPackagePolicy, FailOn, GraphReport,
    INIT_CONFIG_TEMPLATE, ImportEdge, ImportExplanation, ImportResolution, JSON_CONFIG_FILE,
    LayerClassification, LayerClassifier, LayerConfig, LoadedConfig, OnionCryError, Result,
    RulePolicy, Severity, TEMPLATE_ALIAS_BLOCK, ViolationBaseline, apply_inline_suppressions,
    build_glob_set, build_graph_report, build_report, collect_import_edges, load_tsconfig_aliases,
    normalize_path, normalized_package_name, render_generated_alias_block, resolve_against,
};
use globset::Glob;
use jsonc_parser::{ParseOptions, parse_to_serde_value};
use schemars::schema_for;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Copy, Debug)]
pub struct CheckOptions<'a> {
    pub explicit_config: Option<&'a Path>,
    pub fail_on: FailOn,
    pub baseline_path: Option<&'a Path>,
    pub write_baseline: bool,
    pub no_baseline: bool,
    pub scope_files: Option<&'a [PathBuf]>,
}

#[derive(Debug)]
pub struct CheckOutcome {
    pub report: CheckReport,
    pub baseline_write: Option<BaselineWrite>,
    pub baseline_warning: Option<BaselineWarning>,
    pub skipped_scope_paths: Vec<PathBuf>,
    pub project_root: PathBuf,
}

#[derive(Debug)]
pub struct BaselineWrite {
    pub path: PathBuf,
    pub entry_count: usize,
}

#[derive(Debug)]
pub struct BaselineWarning {
    pub path: PathBuf,
    pub stale_entry_count: usize,
}

#[derive(Debug)]
struct LoadedBaseline {
    path: PathBuf,
    baseline: ViolationBaseline,
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
    init_config_with_options(cwd, force, None)
}

pub fn init_config_with_options(
    cwd: &Path,
    force: bool,
    from_tsconfig: Option<&Path>,
) -> Result<PathBuf> {
    let path = cwd.join(DEFAULT_CONFIG_FILE);
    if path.exists() && !force {
        return Err(OnionCryError::ConfigAlreadyExists { path });
    }

    // Generate the template (and fail on tsconfig errors) before touching the config file.
    let contents = match from_tsconfig {
        Some(tsconfig_path) => {
            let generated = load_tsconfig_aliases(cwd, cwd, tsconfig_path)?;
            INIT_CONFIG_TEMPLATE.replace(
                TEMPLATE_ALIAS_BLOCK,
                &render_generated_alias_block(&generated),
            )
        }
        None => INIT_CONFIG_TEMPLATE.to_string(),
    };

    fs::write(&path, contents).map_err(|source| OnionCryError::WriteConfig {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

pub fn render_config_schema_json() -> Result<String> {
    serde_json::to_string_pretty(&schema_for!(Config))
        .map_err(|source| OnionCryError::RenderSchema { source })
}

pub fn write_config_schema(cwd: &Path, path: &Path) -> Result<PathBuf> {
    let schema_json = render_config_schema_json()?;
    let resolved_path = resolve_against(cwd, path);
    if let Some(parent) = resolved_path.parent() {
        fs::create_dir_all(parent).map_err(|source| OnionCryError::WriteSchema {
            path: resolved_path.clone(),
            source,
        })?;
    }
    fs::write(&resolved_path, format!("{schema_json}\n")).map_err(|source| {
        OnionCryError::WriteSchema {
            path: resolved_path.clone(),
            source,
        }
    })?;
    Ok(resolved_path)
}

pub fn run_check(
    cwd: &Path,
    explicit_config: Option<&Path>,
    fail_on: FailOn,
) -> Result<CheckReport> {
    Ok(run_check_with_options(
        cwd,
        CheckOptions {
            explicit_config,
            fail_on,
            baseline_path: None,
            write_baseline: false,
            no_baseline: false,
            scope_files: None,
        },
    )?
    .report)
}

pub fn run_check_with_options(cwd: &Path, options: CheckOptions<'_>) -> Result<CheckOutcome> {
    let loaded = load_config(cwd, options.explicit_config)?;
    let rule_policy = RulePolicy::new(&loaded.config)?;
    let files = select_files(&loaded)?;
    let project_root = loaded.project_root()?;
    let edges = collect_import_edges(&loaded, &project_root, &files)?;
    let rule_context = RuleCollectionContext {
        loaded: &loaded,
        project_root: &project_root,
        files: &files,
        edges: &edges,
        rule_policy: &rule_policy,
    };
    let mut violations = collect_rule_violations(&rule_context)?;
    violations = apply_inline_suppressions(&project_root, &files, &rule_policy, violations)?;
    let baseline_warning = if options.no_baseline {
        None
    } else {
        match load_violation_baseline(cwd, &loaded, options.baseline_path) {
            Ok(Some(loaded_baseline)) => {
                let application = loaded_baseline.baseline.apply(&project_root, violations);
                violations = application.violations;
                // Stale entries are being removed by the rewrite itself, so the
                // "rerun --write-baseline" warning would be noise on a write run.
                (application.stale_entry_count > 0 && !options.write_baseline).then_some(
                    BaselineWarning {
                        path: loaded_baseline.path,
                        stale_entry_count: application.stale_entry_count,
                    },
                )
            }
            Ok(None) => None,
            // A rewrite replaces a missing or version-incompatible baseline instead
            // of failing on the file it is about to overwrite.
            Err(
                OnionCryError::MissingBaseline { .. }
                | OnionCryError::UnsupportedBaselineVersion { .. },
            ) if options.write_baseline => None,
            Err(error) => return Err(error),
        }
    };
    let baseline_write = if options.write_baseline {
        Some(write_violation_baseline(
            cwd,
            &loaded,
            &project_root,
            options.baseline_path,
            &violations,
        )?)
    } else {
        None
    };
    let skipped_scope_paths = if let Some(scope_paths) = options.scope_files {
        let scope = resolve_report_scope(cwd, &files, scope_paths);
        violations.retain(|violation| {
            violation.cycle_path.is_some() || scope.files.contains(&violation.file)
        });
        scope.skipped_paths
    } else {
        Vec::new()
    };

    Ok(CheckOutcome {
        report: build_report(files.len(), &violations, options.fail_on),
        baseline_write,
        baseline_warning,
        skipped_scope_paths,
        project_root,
    })
}

struct ReportScope {
    files: std::collections::BTreeSet<String>,
    skipped_paths: Vec<PathBuf>,
}

fn resolve_report_scope(cwd: &Path, universe: &[PathBuf], scope_paths: &[PathBuf]) -> ReportScope {
    let mut scope_files = std::collections::BTreeSet::new();
    let mut skipped_paths = Vec::new();

    for path in scope_paths {
        let resolved = normalize_path(&resolve_against(cwd, path));
        if universe.binary_search(&resolved).is_ok() {
            scope_files.insert(resolved.display().to_string());
        } else {
            skipped_paths.push(path.clone());
        }
    }

    ReportScope {
        files: scope_files,
        skipped_paths,
    }
}

fn load_violation_baseline(
    cwd: &Path,
    loaded: &LoadedConfig,
    explicit_baseline_path: Option<&Path>,
) -> Result<Option<LoadedBaseline>> {
    let path = baseline_path(cwd, loaded, explicit_baseline_path);
    if !path.exists() {
        return if explicit_baseline_path.is_some() {
            Err(OnionCryError::MissingBaseline { path })
        } else {
            Ok(None)
        };
    }

    let contents = fs::read_to_string(&path).map_err(|source| OnionCryError::ReadBaseline {
        path: path.clone(),
        source,
    })?;
    let baseline = serde_json::from_str::<ViolationBaseline>(&contents).map_err(|source| {
        OnionCryError::ParseBaseline {
            path: path.clone(),
            source,
        }
    })?;
    if baseline.version != crate::baseline::BASELINE_VERSION {
        return Err(OnionCryError::UnsupportedBaselineVersion {
            path,
            version: baseline.version,
        });
    }

    Ok(Some(LoadedBaseline { path, baseline }))
}

fn write_violation_baseline(
    cwd: &Path,
    loaded: &LoadedConfig,
    project_root: &Path,
    explicit_baseline_path: Option<&Path>,
    violations: &[crate::Violation],
) -> Result<BaselineWrite> {
    let path = baseline_path(cwd, loaded, explicit_baseline_path);
    let baseline = ViolationBaseline::from_violations(project_root, violations);
    let json = serde_json::to_string_pretty(&baseline)
        .map_err(|source| OnionCryError::RenderBaseline { source })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| OnionCryError::WriteBaseline {
            path: path.clone(),
            source,
        })?;
    }
    fs::write(&path, format!("{json}\n")).map_err(|source| OnionCryError::WriteBaseline {
        path: path.clone(),
        source,
    })?;

    Ok(BaselineWrite {
        path,
        entry_count: baseline.entries.len(),
    })
}

fn baseline_path(
    cwd: &Path,
    loaded: &LoadedConfig,
    explicit_baseline_path: Option<&Path>,
) -> PathBuf {
    explicit_baseline_path.map_or_else(
        || loaded.config_dir.join(DEFAULT_BASELINE_FILE),
        |path| resolve_against(cwd, path),
    )
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
    let rule_context = RuleCollectionContext {
        loaded: &loaded,
        project_root: &project_root,
        files: &files,
        edges: &edges,
        rule_policy: &rule_policy,
    };
    let violations = collect_rule_violations(&rule_context)?
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

pub fn run_graph(cwd: &Path, explicit_config: Option<&Path>) -> Result<GraphReport> {
    let loaded = load_config(cwd, explicit_config)?;
    let files = select_files(&loaded)?;
    let project_root = loaded.project_root()?;
    let edges = collect_import_edges(&loaded, &project_root, &files)?;

    build_graph_report(&loaded, &project_root, &files, &edges)
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
