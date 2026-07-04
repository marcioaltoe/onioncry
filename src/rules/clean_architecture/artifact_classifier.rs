use crate::{artifact_role_folder, is_core_clean_artifact_role};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub(super) struct CleanArtifact {
    pub(super) role: String,
    pub(super) folder_index: Option<usize>,
}

pub(super) struct CleanArtifactClassifier {
    folder_layers: BTreeMap<String, BTreeSet<String>>,
    suffix_roles: BTreeMap<String, Vec<String>>,
}

impl CleanArtifactClassifier {
    pub(super) fn new(
        folder_layers: BTreeMap<String, BTreeSet<String>>,
        suffix_roles: BTreeMap<String, Vec<String>>,
    ) -> Self {
        Self {
            folder_layers,
            suffix_roles,
        }
    }

    pub(super) fn classify(&self, components: &[String]) -> Option<CleanArtifact> {
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
}
