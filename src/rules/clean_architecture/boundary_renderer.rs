use crate::display_path_components;
use std::collections::BTreeMap;

pub(super) struct CleanArtifactBoundaryRenderer {
    source_prefix: Vec<String>,
    context_root: Vec<String>,
    artifact_folders_by_layer: BTreeMap<String, Vec<String>>,
}

impl CleanArtifactBoundaryRenderer {
    pub(super) fn new(
        source_prefix: Vec<String>,
        context_root: Vec<String>,
        artifact_folders_by_layer: BTreeMap<String, Vec<String>>,
    ) -> Self {
        Self {
            source_prefix,
            context_root,
            artifact_folders_by_layer,
        }
    }

    pub(super) fn layer_artifact_boundary(
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

    pub(super) fn grouped_artifact_boundary(&self, layer: &str, role: &str) -> String {
        format!("{}/<group>", self.expected_boundary(None, layer, role))
    }

    pub(super) fn expected_boundary(
        &self,
        context: Option<&str>,
        layer: &str,
        role: &str,
    ) -> String {
        self.expected_boundary_with_group(context, layer, role, None)
    }

    pub(super) fn expected_boundary_with_group(
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
