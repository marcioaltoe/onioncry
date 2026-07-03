use crate::*;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

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

pub(crate) fn collect_clean_artifact_placement_violations(
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
