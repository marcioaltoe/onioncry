use crate::rules::{
    collect_clean_artifact_placement_violations, collect_concrete_dependency_violations,
    collect_context_cycle_violations, collect_context_violations,
    collect_external_package_violations, collect_feature_envy_violations,
    collect_feature_system_adapter_contract_violations,
    collect_feature_system_dependency_flow_violations, collect_feature_system_layout_violations,
    collect_feature_system_public_api_violations, collect_feature_system_query_contract_violations,
    collect_framework_in_core_violations, collect_global_slice_artifact_violations,
    collect_layer_violations, collect_outer_data_format_violations, collect_path_naming_violations,
    collect_public_surface_reexport_violations, collect_shotgun_surgery_violations,
    collect_test_placement_violations, collect_unowned_schema_import_violations,
    collect_vertical_shared_layer_artifact_violations,
    collect_vertical_slice_entry_point_violations,
    collect_vertical_slice_internal_import_violations,
};
use crate::*;
use globset::Glob;
use jsonc_parser::{ParseOptions, parse_to_serde_value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
