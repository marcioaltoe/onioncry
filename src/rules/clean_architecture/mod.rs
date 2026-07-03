mod artifact_classifier;
mod artifact_policy;
mod boundary_renderer;
mod file_index;
mod location;

use artifact_policy::CleanArtifactPlacementPolicy;
use file_index::CleanArtifactPlacementFileIndex;

use crate::rules::catalog::RULE_CLEAN_ARTIFACT_PLACEMENT;
use crate::{LoadedConfig, Result, RulePolicy, Severity, Violation};
use std::path::{Path, PathBuf};

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
