use crate::rules::catalog::RULE_PATH_NAMING;
use crate::{
    DEFAULT_COLLECTION_DIRECTORIES, DEFAULT_FEATURE_ROOTS, DEFAULT_IGNORED_PATH_DIRECTORIES,
    DEFAULT_LAYER_DIRECTORIES, DEFAULT_SUFFIXES_BY_COLLECTION, Result, RulePolicy, RuleSetting,
    Severity, Violation, is_kebab_case_file_name, is_kebab_case_name, path_has_prefix_components,
    path_roots, plural_like, project_relative_components, singular_directory_name,
    source_file_stem, stem_matches_collection_suffix, string_set_option, string_vec_option,
    suffix_map_option,
};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

struct PathNamingPolicy {
    collection_directories: HashSet<String>,
    singular_collection_directories: BTreeMap<String, String>,
    feature_roots: Vec<Vec<String>>,
    layer_directories: HashSet<String>,
    ignored_directories: HashSet<String>,
    suffixes_by_collection: BTreeMap<String, Vec<String>>,
}

pub(crate) fn collect_path_naming_violations(
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    for file in files {
        let rule_setting = rule_policy.effective_rule(RULE_PATH_NAMING, project_root, file);
        if rule_setting.severity == Severity::Off {
            continue;
        }
        let policy = PathNamingPolicy::from_rule_setting(&rule_setting)?;
        violations.extend(policy.violations(project_root, file, rule_setting.severity));
    }

    Ok(violations)
}

impl PathNamingPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let collection_directories = string_set_option(
            RULE_PATH_NAMING,
            setting,
            "collectionDirectories",
            DEFAULT_COLLECTION_DIRECTORIES,
        )?;
        let feature_roots = string_vec_option(
            RULE_PATH_NAMING,
            setting,
            "featureRoots",
            DEFAULT_FEATURE_ROOTS,
        )?;
        let layer_directories = string_set_option(
            RULE_PATH_NAMING,
            setting,
            "layerDirectories",
            DEFAULT_LAYER_DIRECTORIES,
        )?;
        let ignored_directories = string_set_option(
            RULE_PATH_NAMING,
            setting,
            "ignoredDirectories",
            DEFAULT_IGNORED_PATH_DIRECTORIES,
        )?;
        let suffixes_by_collection = suffix_map_option(
            RULE_PATH_NAMING,
            setting,
            "suffixes",
            DEFAULT_SUFFIXES_BY_COLLECTION,
        )?;
        let singular_collection_directories = collection_directories
            .iter()
            .map(|directory| (singular_directory_name(directory), directory.clone()))
            .filter(|(singular, plural)| singular != plural)
            .collect();

        Ok(Self {
            collection_directories,
            singular_collection_directories,
            feature_roots: path_roots(feature_roots),
            layer_directories,
            ignored_directories,
            suffixes_by_collection,
        })
    }

    fn violations(&self, project_root: &Path, file: &Path, severity: Severity) -> Vec<Violation> {
        let components = project_relative_components(project_root, file);
        let mut violations = Vec::new();

        for (index, directory) in components
            .iter()
            .take(components.len().saturating_sub(1))
            .enumerate()
        {
            if self.ignored_directories.contains(directory) {
                continue;
            }
            if !is_kebab_case_name(directory) {
                violations.push(Violation::path_naming(
                    file,
                    severity,
                    format!("directory segment {directory:?} should use kebab-case"),
                    "rename the directory segment to lowercase kebab-case".to_string(),
                ));
            }
            if let Some(plural) = self.singular_collection_directories.get(directory) {
                violations.push(Violation::path_naming(
                    file,
                    severity,
                    format!("collection directory {directory:?} should be plural"),
                    format!("rename {directory:?} to {plural:?}"),
                ));
            }
            if directory == "infrastructure" && !self.layer_directories.contains(directory) {
                violations.push(Violation::path_naming(
                    file,
                    severity,
                    "layer directory \"infrastructure\" should use the configured layer vocabulary"
                        .to_string(),
                    "use \"infra\" or configure repo/path-naming.layerDirectories".to_string(),
                ));
            }
            if self.is_feature_segment(&components, index) && plural_like(directory) {
                violations.push(Violation::path_naming(
                    file,
                    severity,
                    format!("feature directory {directory:?} should be singular"),
                    format!(
                        "rename {directory:?} to a singular kebab-case feature or context name"
                    ),
                ));
            }
        }

        if let Some(file_name) = components.last() {
            if !is_kebab_case_file_name(file_name) {
                violations.push(Violation::path_naming(
                    file,
                    severity,
                    format!("file name {file_name:?} should use kebab-case"),
                    "rename the file to lowercase kebab-case while keeping any configured suffix"
                        .to_string(),
                ));
            }
            if let Some((collection, suffixes)) = self.nearest_suffix_collection(&components) {
                let stem = source_file_stem(file_name);
                if stem != "index" && !stem_matches_collection_suffix(stem, suffixes) {
                    violations.push(Violation::path_naming(
                        file,
                        severity,
                        format!("files in {collection:?} should use a configured suffix"),
                        format!(
                            "rename the file so its stem ends with one of: {} (optionally followed by .test or .spec)",
                            suffixes.join(", ")
                        ),
                    ));
                }
            }
        }

        violations
    }

    fn is_feature_segment(&self, components: &[String], index: usize) -> bool {
        self.feature_roots.iter().any(|root| {
            index == root.len()
                && path_has_prefix_components(components, root)
                && !self.collection_directories.contains(&components[index])
                && !self.layer_directories.contains(&components[index])
                && !self.ignored_directories.contains(&components[index])
        })
    }

    fn nearest_suffix_collection<'a>(
        &'a self,
        components: &[String],
    ) -> Option<(&'a str, &'a [String])> {
        components.iter().rev().skip(1).find_map(|component| {
            self.suffixes_by_collection
                .get_key_value(component)
                .map(|(collection, suffixes)| (collection.as_str(), suffixes.as_slice()))
        })
    }
}
