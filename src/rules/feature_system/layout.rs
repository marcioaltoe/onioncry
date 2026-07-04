use crate::rules::catalog::RULE_FEATURE_SYSTEM_LAYOUT;
use crate::{
    DEFAULT_ALLOWED_SHARED_COMPONENT_ROOTS, DEFAULT_COMPONENT_DIRECTORIES,
    DEFAULT_FEATURE_SYSTEM_OPTIONAL_DIRECTORIES, DEFAULT_FEATURE_SYSTEM_REQUIRED_DIRECTORIES,
    DEFAULT_FEATURE_SYSTEM_ROOT_INDEX_FILE, DEFAULT_LEGACY_FEATURE_ROOTS,
    DEFAULT_SURFACE_CSS_NAME_TEMPLATE, DEFAULT_SYSTEMS_ROOTS, Result, RuleSetting, Severity,
    Violation, display_path_components, display_root, is_component_source_file,
    path_from_components, path_has_prefix_components, path_roots, path_under_any_root,
    project_relative_components, sorted_strings, string_option, string_set_option,
    string_vec_option,
};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) struct FeatureSystemLayoutPolicy {
    systems_roots: Vec<Vec<String>>,
    required_directories: HashSet<String>,
    optional_directories: HashSet<String>,
    root_index_file: String,
    allowed_shared_component_roots: Vec<Vec<String>>,
    legacy_roots: Vec<Vec<String>>,
    component_directories: HashSet<String>,
    surface_css_name_template: String,
}

struct FeatureSystem {
    domain: String,
    path: PathBuf,
    representative_file: PathBuf,
}

impl FeatureSystemLayoutPolicy {
    pub(super) fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let systems_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "systemsRoots",
            DEFAULT_SYSTEMS_ROOTS,
        )?;
        let required_directories = string_set_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "requiredDirectories",
            DEFAULT_FEATURE_SYSTEM_REQUIRED_DIRECTORIES,
        )?;
        let optional_directories = string_set_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "optionalDirectories",
            DEFAULT_FEATURE_SYSTEM_OPTIONAL_DIRECTORIES,
        )?;
        let root_index_file = string_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "rootIndexFile",
            DEFAULT_FEATURE_SYSTEM_ROOT_INDEX_FILE,
        )?;
        let allowed_shared_component_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "allowedSharedComponentRoots",
            DEFAULT_ALLOWED_SHARED_COMPONENT_ROOTS,
        )?;
        let legacy_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "legacyRoots",
            DEFAULT_LEGACY_FEATURE_ROOTS,
        )?;
        let component_directories = string_set_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "componentDirectories",
            DEFAULT_COMPONENT_DIRECTORIES,
        )?;
        let surface_css_name_template = string_option(
            RULE_FEATURE_SYSTEM_LAYOUT,
            setting,
            "surfaceCssNameTemplate",
            DEFAULT_SURFACE_CSS_NAME_TEMPLATE,
        )?;

        Ok(Self {
            systems_roots: path_roots(systems_roots),
            required_directories,
            optional_directories,
            root_index_file,
            allowed_shared_component_roots: path_roots(allowed_shared_component_roots),
            legacy_roots: path_roots(legacy_roots),
            component_directories,
            surface_css_name_template,
        })
    }

    pub(super) fn violations(
        &self,
        project_root: &Path,
        files: &[PathBuf],
        severity: Severity,
    ) -> Vec<Violation> {
        let systems = self.discover_systems(project_root, files);
        let _recognized_optional_directories = &self.optional_directories;
        let mut violations = Vec::new();

        for system in systems.values() {
            for directory in sorted_strings(&self.required_directories) {
                if !system.path.join(&directory).is_dir() {
                    violations.push(Violation::feature_system_layout(
                        &system.representative_file,
                        severity,
                        format!(
                            "feature system {:?} is missing required {directory}/ directory",
                            system.domain
                        ),
                        format!("add {directory}/ under {}", system.path.display()),
                    ));
                }
            }
            if !system.path.join(&self.root_index_file).is_file() {
                violations.push(Violation::feature_system_layout(
                    &system.representative_file,
                    severity,
                    format!(
                        "feature system {:?} is missing root {}",
                        system.domain, self.root_index_file
                    ),
                    format!(
                        "add {} under {}",
                        self.root_index_file,
                        system.path.display()
                    ),
                ));
            }
        }

        for file in files {
            if let Some((legacy_root, domain)) = self.legacy_feature(project_root, file) {
                violations.push(Violation::feature_system_layout(
                    file,
                    severity,
                    format!(
                        "legacy feature root {}/{} should use a feature system",
                        display_path_components(&legacy_root),
                        domain
                    ),
                    format!(
                        "move feature code to {}/{}",
                        display_root(&self.systems_roots),
                        domain
                    ),
                ));
            }
            if self.is_feature_component_outside_allowed_roots(project_root, file) {
                violations.push(Violation::feature_system_layout(
                    file,
                    severity,
                    "feature-specific frontend component is outside a feature system".to_string(),
                    format!(
                        "move this component under {}/<domain>/components or an allowed shared UI root",
                        display_root(&self.systems_roots)
                    ),
                ));
            }
            if let Some((domain, expected_file_name, root_level)) =
                self.surface_css_status(project_root, file)
            {
                if !root_level {
                    violations.push(Violation::feature_system_layout(
                        file,
                        severity,
                        format!(
                            "surface CSS for feature system {domain:?} must live at the system root"
                        ),
                        format!(
                            "move this CSS file to {}/{domain}/{expected_file_name}",
                            display_root(&self.systems_roots)
                        ),
                    ));
                } else if file.file_name().and_then(|file_name| file_name.to_str())
                    != Some(expected_file_name.as_str())
                {
                    violations.push(Violation::feature_system_layout(
                        file,
                        severity,
                        format!(
                            "surface CSS for feature system {domain:?} should be named {expected_file_name:?}"
                        ),
                        format!("rename this file to {expected_file_name}"),
                    ));
                }
            }
        }

        violations
    }

    fn discover_systems(
        &self,
        project_root: &Path,
        files: &[PathBuf],
    ) -> BTreeMap<PathBuf, FeatureSystem> {
        let mut systems = BTreeMap::new();
        for file in files {
            let components = project_relative_components(project_root, file);
            let Some((root, domain)) = self.system_root_and_domain(&components) else {
                continue;
            };
            let mut system_components = root.to_vec();
            system_components.push(domain.to_string());
            let system_path = project_root.join(path_from_components(&system_components));
            systems
                .entry(system_path.clone())
                .or_insert_with(|| FeatureSystem {
                    domain: domain.to_string(),
                    path: system_path,
                    representative_file: file.clone(),
                });
        }
        systems
    }

    fn system_root_and_domain<'a>(
        &'a self,
        components: &'a [String],
    ) -> Option<(&'a [String], &'a str)> {
        self.systems_roots.iter().find_map(|root| {
            (components.len() > root.len() && path_has_prefix_components(components, root))
                .then(|| (root.as_slice(), components[root.len()].as_str()))
        })
    }

    fn legacy_feature(&self, project_root: &Path, file: &Path) -> Option<(Vec<String>, String)> {
        let components = project_relative_components(project_root, file);
        self.legacy_roots.iter().find_map(|root| {
            (components.len() > root.len() && path_has_prefix_components(&components, root))
                .then(|| (root.clone(), components[root.len()].clone()))
        })
    }

    fn is_feature_component_outside_allowed_roots(&self, project_root: &Path, file: &Path) -> bool {
        if !is_component_source_file(file) {
            return false;
        }
        let components = project_relative_components(project_root, file);
        if !components
            .iter()
            .any(|component| self.component_directories.contains(component))
        {
            return false;
        }
        if self.system_root_and_domain(&components).is_some()
            || path_under_any_root(&components, &self.allowed_shared_component_roots)
        {
            return false;
        }

        true
    }

    fn surface_css_status(
        &self,
        project_root: &Path,
        file: &Path,
    ) -> Option<(String, String, bool)> {
        if !file
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .is_some_and(|file_name| file_name.ends_with(".css"))
        {
            return None;
        }
        let components = project_relative_components(project_root, file);
        let (root, domain) = self.system_root_and_domain(&components)?;
        let expected_file_name = self.surface_css_name_template.replace("{domain}", domain);
        let root_level = components.len() == root.len() + 2;
        Some((domain.to_string(), expected_file_name, root_level))
    }
}
