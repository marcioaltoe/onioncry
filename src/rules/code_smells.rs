use crate::rules::catalog::{RULE_FEATURE_ENVY, RULE_NO_CONCRETE_DEPENDENCY, RULE_SHOTGUN_SURGERY};
use crate::{
    ContextClassification, ContextClassifier, ImportEdge, ImportResolution, LayerClassification,
    LayerClassifier, LoadedConfig, Result, RulePolicy, Severity, Violation, bool_option,
    normalize_path, path_ends_with_any, path_has_any_segment, project_relative_display,
    string_set_option, string_vec_option, usize_option,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

struct FeatureImportCounts {
    source_context: String,
    own_context_count: usize,
    other_context_counts: BTreeMap<String, usize>,
}

pub(crate) fn collect_concrete_dependency_violations(
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

pub(crate) fn collect_feature_envy_violations(
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

pub(crate) fn collect_shotgun_surgery_violations(
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

impl FeatureImportCounts {
    fn new(source_context: &str) -> Self {
        Self {
            source_context: source_context.to_string(),
            own_context_count: 0,
            other_context_counts: BTreeMap::new(),
        }
    }
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
