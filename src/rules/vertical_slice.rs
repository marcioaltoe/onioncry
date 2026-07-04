use crate::rules::catalog::{
    RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT, RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS,
    RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS, RULE_VERTICAL_SLICE_ENTRY_POINT,
};
use crate::{
    ImportEdge, ImportResolution, LoadedConfig, OnionCryError, Result, RulePolicy, Severity,
    VerticalSliceConfig, Violation, artifact_role_folder, display_path_components,
    is_test_file_name, path_components, path_has_prefix_components, project_relative_components,
};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct VerticalSlicePolicy {
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
pub(crate) struct SliceLocation {
    pub(crate) slice: String,
    pub(crate) slice_path: String,
    pub(crate) relative_file: Vec<String>,
}

pub(crate) fn collect_vertical_slice_internal_import_violations(
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

pub(crate) fn collect_global_slice_artifact_violations(
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

pub(crate) fn collect_vertical_slice_entry_point_violations(
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

pub(crate) fn collect_vertical_shared_layer_artifact_violations(
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

impl VerticalSlicePolicy {
    pub(crate) fn from_config(config: &VerticalSliceConfig) -> Self {
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

    pub(crate) fn slice_location(&self, project_root: &Path, file: &Path) -> Option<SliceLocation> {
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

    pub(crate) fn is_public_surface_location(&self, location: &SliceLocation) -> bool {
        self.public_surface_label(location).is_some()
    }

    pub(crate) fn public_surface_label(&self, location: &SliceLocation) -> Option<String> {
        let relative_file = &location.relative_file;
        self.public_surface.iter().find_map(|surface| {
            (relative_file == surface
                || (!surface.is_empty()
                    && path_has_prefix_components(relative_file.as_slice(), surface.as_slice())
                    && relative_file.len() > surface.len()))
            .then(|| display_path_components(surface))
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
