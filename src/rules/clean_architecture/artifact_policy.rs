use super::artifact_classifier::{CleanArtifact, CleanArtifactClassifier};
use super::boundary_renderer::CleanArtifactBoundaryRenderer;
use super::file_index::CleanArtifactPlacementFileIndex;
use super::location::CleanArtifactLocationPolicy;
use crate::{
    CleanArchitectureConfig, is_index_file_name, path_components, project_relative_components,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;

pub(super) struct CleanArtifactPlacementPolicy {
    classifier: CleanArtifactClassifier,
    location: CleanArtifactLocationPolicy,
    boundary_renderer: CleanArtifactBoundaryRenderer,
    artifact_folders_by_layer: BTreeMap<String, Vec<String>>,
    folder_layers: BTreeMap<String, BTreeSet<String>>,
    grouped_artifact_folders: HashSet<String>,
}

pub(super) struct CleanArtifactFinding {
    pub(super) role: String,
    pub(super) expected_layer: String,
    pub(super) expected_boundary: String,
}

impl CleanArtifactPlacementPolicy {
    pub(super) fn from_config(config: &CleanArchitectureConfig) -> Self {
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
            classifier: CleanArtifactClassifier::new(
                folder_layers.clone(),
                config.artifact_suffixes.clone(),
            ),
            location: CleanArtifactLocationPolicy::new(
                source_prefix.clone(),
                context_root.clone(),
                layer_aliases,
                folder_layers.clone(),
            ),
            boundary_renderer: CleanArtifactBoundaryRenderer::new(
                source_prefix,
                context_root,
                config.artifact_folders.clone(),
            ),
            artifact_folders_by_layer: config.artifact_folders.clone(),
            folder_layers,
            grouped_artifact_folders: config.grouped_artifact_folders.iter().cloned().collect(),
        }
    }

    pub(super) fn finding(
        &self,
        project_root: &Path,
        file: &Path,
        file_index: &CleanArtifactPlacementFileIndex,
    ) -> Option<CleanArtifactFinding> {
        let components = project_relative_components(project_root, file);
        if components.is_empty() {
            return None;
        }
        let artifact = match self.classifier.classify(&components) {
            Some(artifact) => artifact,
            None => return self.layer_direct_folder_finding(&components, file_index),
        };
        let current_layer = self.location.first_layer(&components);
        let expected_layer = self.expected_layer(&artifact.role, current_layer.as_deref())?;

        if let Some((context, layer)) = self.location.context_first_location(&components) {
            if layer == expected_layer {
                return None;
            }
            return Some(CleanArtifactFinding {
                role: artifact.role.clone(),
                expected_boundary: self.boundary_renderer.expected_boundary(
                    Some(&context),
                    &expected_layer,
                    &artifact.role,
                ),
                expected_layer,
            });
        }

        if let Some(context) = self.location.context_missing_layer(&components) {
            return Some(CleanArtifactFinding {
                role: artifact.role.clone(),
                expected_boundary: self.boundary_renderer.expected_boundary(
                    Some(&context),
                    &expected_layer,
                    &artifact.role,
                ),
                expected_layer,
            });
        }

        if let Some((layer, layer_index)) = self.location.first_layer_with_index(&components) {
            if layer != expected_layer {
                let context = self.location.context_before_layer(&components, layer_index);
                return Some(CleanArtifactFinding {
                    role: artifact.role.clone(),
                    expected_boundary: self.boundary_renderer.expected_boundary(
                        context,
                        &expected_layer,
                        &artifact.role,
                    ),
                    expected_layer,
                });
            }
            if self
                .location
                .is_contextless_base_layer(&components, layer_index)
            {
                if self.is_flat_grouped_artifact(&components, layer_index, &artifact, file_index) {
                    return Some(CleanArtifactFinding {
                        role: artifact.role.clone(),
                        expected_boundary: self
                            .boundary_renderer
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
                if let Some(context) = self.location.layer_first_context_candidate(
                    &components,
                    layer_index,
                    artifact.folder_index,
                ) {
                    return Some(CleanArtifactFinding {
                        role: artifact.role.clone(),
                        expected_boundary: self.boundary_renderer.expected_boundary(
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
                expected_boundary: self.boundary_renderer.expected_boundary(
                    self.location.context_before_layer(&components, layer_index),
                    &expected_layer,
                    &artifact.role,
                ),
                expected_layer,
            });
        }

        let context = self.location.context_after_source_prefix(&components);
        Some(CleanArtifactFinding {
            role: artifact.role.clone(),
            expected_boundary: self.boundary_renderer.expected_boundary(
                context,
                &expected_layer,
                &artifact.role,
            ),
            expected_layer,
        })
    }

    fn layer_direct_folder_finding(
        &self,
        components: &[String],
        file_index: &CleanArtifactPlacementFileIndex,
    ) -> Option<CleanArtifactFinding> {
        let (layer, layer_index) = self.location.first_layer_with_index(components)?;
        let child_index = layer_index + 1;
        let child = components.get(child_index)?;
        if is_index_file_name(child) || self.location.layer_for_segment(child).is_some() {
            return None;
        }
        let folders = self.artifact_folders_by_layer.get(&layer)?;
        if folders.is_empty() || folders.iter().any(|folder| folder == child) {
            return None;
        }

        let context = if self
            .location
            .is_contextless_base_layer(components, layer_index)
        {
            None
        } else if let Some((context, context_layer)) =
            self.location.context_first_location(components)
        {
            (context_layer == layer).then_some(context)
        } else {
            self.location
                .context_before_layer(components, layer_index)
                .map(str::to_string)
        };
        let group =
            (file_index.direct_file_count(components, child_index) > 1).then_some(child.as_str());

        Some(CleanArtifactFinding {
            role: layer.clone(),
            expected_boundary: self.boundary_renderer.layer_artifact_boundary(
                context.as_deref(),
                &layer,
                group,
            ),
            expected_layer: layer,
        })
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
            || self.location.is_layer_or_artifact_segment(child)
            || is_index_file_name(child)
        {
            return None;
        }

        let group =
            (file_index.direct_file_count(components, child_index) > 1).then_some(child.as_str());
        Some(self.boundary_renderer.expected_boundary_with_group(
            None,
            expected_layer,
            &artifact.role,
            group,
        ))
    }
}
