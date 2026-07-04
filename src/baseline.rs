use crate::{Violation, project_relative_display};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const DEFAULT_BASELINE_FILE: &str = ".onioncry-baseline.json";
pub const BASELINE_VERSION: u8 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViolationBaseline {
    pub version: u8,
    pub entries: Vec<BaselineEntry>,
}

#[derive(Debug)]
pub struct BaselineApplication {
    pub violations: Vec<Violation>,
    pub stale_entry_count: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineEntry {
    pub file: String,
    pub rule: String,
    pub target: String,
    pub count: usize,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct BaselineFingerprint {
    file: String,
    rule: String,
    target: String,
}

impl ViolationBaseline {
    pub fn from_violations(project_root: &Path, violations: &[Violation]) -> Self {
        let mut counts = BTreeMap::<BaselineFingerprint, usize>::new();
        for violation in violations {
            let fingerprint = BaselineFingerprint {
                file: project_relative_display(project_root, Path::new(&violation.file)),
                rule: violation.rule.clone(),
                target: baseline_target(violation),
            };
            *counts.entry(fingerprint).or_default() += 1;
        }

        let entries = counts
            .into_iter()
            .map(|(fingerprint, count)| BaselineEntry {
                file: fingerprint.file,
                rule: fingerprint.rule,
                target: fingerprint.target,
                count,
            })
            .collect();

        Self {
            version: BASELINE_VERSION,
            entries,
        }
    }

    pub fn apply(
        &self,
        project_root: &Path,
        mut violations: Vec<Violation>,
    ) -> BaselineApplication {
        let mut remaining_counts = BTreeMap::<BaselineFingerprint, usize>::new();
        for entry in &self.entries {
            *remaining_counts
                .entry(BaselineFingerprint::from_entry(entry))
                .or_default() += entry.count;
        }

        let mut matched_fingerprints = BTreeSet::<BaselineFingerprint>::new();
        for violation in &mut violations {
            let fingerprint = BaselineFingerprint::from_violation(project_root, violation);
            let Some(remaining_count) = remaining_counts.get_mut(&fingerprint) else {
                continue;
            };
            matched_fingerprints.insert(fingerprint);
            if *remaining_count > 0 {
                violation.baselined = true;
                *remaining_count -= 1;
            }
        }

        let stale_entry_count = self
            .entries
            .iter()
            .filter(|entry| !matched_fingerprints.contains(&BaselineFingerprint::from_entry(entry)))
            .count();

        BaselineApplication {
            violations,
            stale_entry_count,
        }
    }
}

impl BaselineFingerprint {
    fn from_entry(entry: &BaselineEntry) -> Self {
        Self {
            file: entry.file.clone(),
            rule: entry.rule.clone(),
            target: entry.target.clone(),
        }
    }

    fn from_violation(project_root: &Path, violation: &Violation) -> Self {
        Self {
            file: project_relative_display(project_root, Path::new(&violation.file)),
            rule: violation.rule.clone(),
            target: baseline_target(violation),
        }
    }
}

fn baseline_target(violation: &Violation) -> String {
    violation
        .import_specifier
        .clone()
        .or_else(|| violation.package_name.clone())
        .or_else(|| violation.target_file.clone())
        .or_else(|| violation.cycle_path.as_ref().map(|path| path.join(" -> ")))
        .or_else(|| {
            violation
                .matched_layers
                .as_ref()
                .map(|layers| layers.join(", "))
        })
        .or_else(|| {
            violation
                .matched_contexts
                .as_ref()
                .map(|contexts| contexts.join(", "))
        })
        .or_else(|| {
            boundary_pair(
                violation.from_layer.as_deref(),
                violation.to_layer.as_deref(),
            )
        })
        .or_else(|| {
            boundary_pair(
                violation.from_context.as_deref(),
                violation.to_context.as_deref(),
            )
        })
        .unwrap_or_else(|| "file".to_string())
}

fn boundary_pair(from: Option<&str>, to: Option<&str>) -> Option<String> {
    match (from, to) {
        (Some(from), Some(to)) => Some(format!("{from} -> {to}")),
        (Some(from), None) => Some(from.to_string()),
        (None, Some(to)) => Some(to.to_string()),
        (None, None) => None,
    }
}
