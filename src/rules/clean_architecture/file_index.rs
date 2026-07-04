use crate::{is_index_file_name, is_test_file_name, project_relative_components};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) struct CleanArtifactPlacementFileIndex {
    direct_source_file_counts: BTreeMap<Vec<String>, usize>,
}

impl CleanArtifactPlacementFileIndex {
    pub(super) fn from_files(project_root: &Path, files: &[PathBuf]) -> Self {
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

    pub(super) fn direct_file_count(&self, components: &[String], folder_index: usize) -> usize {
        components
            .get(..=folder_index)
            .and_then(|parent| self.direct_source_file_counts.get(parent))
            .copied()
            .unwrap_or_default()
    }
}
