use crate::*;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

struct FeatureSystemLayoutPolicy {
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

struct FeatureSystemPublicApiPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    allowed_public_entry_points: HashSet<String>,
    reject_wildcard_reexports: bool,
}

struct FeatureSystemDependencyFlowPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    allowed_public_entry_points: HashSet<String>,
    adapter_bridge_files: HashSet<String>,
    allowed_imports: BTreeMap<String, HashSet<String>>,
}

struct FeatureSystemAdapterContractPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    adapter_directory: String,
    adapter_file_name_template: String,
    api_export_name_template: String,
    error_export_name_template: String,
    http_client_names: Vec<String>,
}

struct FeatureSystemQueryContractPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    adapter_directory: String,
    query_keys_file: String,
    query_options_file: String,
}

struct FeatureSystemAdapterInfo {
    domain: String,
    representative_file: PathBuf,
    expected_adapter_file: PathBuf,
    expected_relative_file: String,
}

struct FeatureSystemQueryState {
    domain: String,
    system_path: String,
    representative_file: PathBuf,
    files: Vec<PathBuf>,
    requires_query_keys: bool,
    requires_query_options: bool,
}

pub(crate) struct FeatureSystemLocation {
    pub(crate) domain: String,
    pub(crate) system_path: String,
    pub(crate) relative_file: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FeatureSystemDependencyArea {
    PublicEntry,
    Adapters,
    Lib,
    Hooks,
    Contexts,
    Stores,
    Components,
    Guards,
    Routes,
    Other,
}

pub(crate) fn collect_feature_system_layout_violations(
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting = rule_policy.effective_rule(RULE_FEATURE_SYSTEM_LAYOUT, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemLayoutPolicy::from_rule_setting(&rule_setting)?;

    Ok(policy.violations(project_root, files, rule_setting.severity))
}

pub(crate) fn collect_feature_system_public_api_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_PUBLIC_API, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemPublicApiPolicy::from_rule_setting(&rule_setting)?;

    policy.violations(project_root, files, edges, rule_setting.severity)
}

pub(crate) fn collect_feature_system_dependency_flow_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemDependencyFlowPolicy::from_rule_setting(&rule_setting)?;

    Ok(policy.violations(project_root, edges, rule_setting.severity))
}

pub(crate) fn collect_feature_system_adapter_contract_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemAdapterContractPolicy::from_rule_setting(&rule_setting)?;

    policy.violations(project_root, files, edges, rule_setting.severity)
}

pub(crate) fn collect_feature_system_query_contract_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_QUERY_CONTRACT, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemQueryContractPolicy::from_rule_setting(&rule_setting)?;

    policy.violations(project_root, files, edges, rule_setting.severity)
}

impl FeatureSystemLayoutPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
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

    fn violations(
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

impl FeatureSystemPublicApiPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let systems_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_PUBLIC_API,
            setting,
            "systemsRoots",
            DEFAULT_SYSTEMS_ROOTS,
        )?;
        let route_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_PUBLIC_API,
            setting,
            "routeRoots",
            DEFAULT_ROUTE_ROOTS,
        )?;
        let allowed_public_entry_points = string_set_option(
            RULE_FEATURE_SYSTEM_PUBLIC_API,
            setting,
            "allowedPublicEntryPoints",
            DEFAULT_FEATURE_SYSTEM_PUBLIC_ENTRY_POINTS,
        )?;
        let reject_wildcard_reexports = bool_option(
            RULE_FEATURE_SYSTEM_PUBLIC_API,
            setting,
            "rejectWildcardReExports",
            true,
        )?;

        Ok(Self {
            systems_roots: path_roots(systems_roots),
            route_roots: path_roots(route_roots),
            allowed_public_entry_points,
            reject_wildcard_reexports,
        })
    }

    fn violations(
        &self,
        project_root: &Path,
        files: &[PathBuf],
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        if self.reject_wildcard_reexports {
            for file in files {
                let Some(location) = self.system_location(project_root, file) else {
                    continue;
                };
                if !self.is_public_entry(&location) {
                    continue;
                }
                let source =
                    fs::read_to_string(file).map_err(|source| OnionCryError::ReadSource {
                        path: file.clone(),
                        source,
                    })?;
                if has_wildcard_reexport(&source) {
                    violations.push(Violation::feature_system_public_api_wildcard(
                        file, severity,
                    ));
                }
            }
        }

        for edge in edges {
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            let Some(target_location) = self.system_location(project_root, target) else {
                continue;
            };
            if self.is_public_entry(&target_location) {
                continue;
            }
            if self
                .system_location(project_root, &edge.source)
                .is_some_and(|source_location| {
                    source_location.system_path == target_location.system_path
                })
            {
                continue;
            }

            violations.push(Violation::feature_system_public_api_internal_import(
                edge,
                target,
                &target_location,
                self.is_route_file(project_root, &edge.source),
                severity,
            ));
        }

        Ok(violations)
    }

    fn system_location(&self, project_root: &Path, file: &Path) -> Option<FeatureSystemLocation> {
        let components = project_relative_components(project_root, file);
        self.systems_roots.iter().find_map(|root| {
            if components.len() <= root.len() || !path_has_prefix_components(&components, root) {
                return None;
            }
            let domain = components[root.len()].clone();
            let system_components = &components[..=root.len()];
            let relative_components = &components[root.len() + 1..];
            Some(FeatureSystemLocation {
                domain,
                system_path: display_path_components(system_components),
                relative_file: display_path_components(relative_components),
            })
        })
    }

    fn is_public_entry(&self, location: &FeatureSystemLocation) -> bool {
        self.allowed_public_entry_points
            .contains(&location.relative_file)
    }

    fn is_route_file(&self, project_root: &Path, file: &Path) -> bool {
        let components = project_relative_components(project_root, file);
        path_under_any_root(&components, &self.route_roots)
    }
}

impl FeatureSystemDependencyFlowPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let systems_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
            setting,
            "systemsRoots",
            DEFAULT_SYSTEMS_ROOTS,
        )?;
        let route_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
            setting,
            "routeRoots",
            DEFAULT_ROUTE_ROOTS,
        )?;
        let allowed_public_entry_points = string_set_option(
            RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
            setting,
            "allowedPublicEntryPoints",
            DEFAULT_FEATURE_SYSTEM_PUBLIC_ENTRY_POINTS,
        )?;
        let adapter_bridge_files = string_set_option(
            RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
            setting,
            "adapterBridgeFiles",
            DEFAULT_FEATURE_SYSTEM_ADAPTER_BRIDGE_FILES,
        )?;
        let allowed_imports = string_set_map_option(
            RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
            setting,
            "allowedImports",
            DEFAULT_FEATURE_SYSTEM_ALLOWED_IMPORTS,
        )?;

        Ok(Self {
            systems_roots: path_roots(systems_roots),
            route_roots: path_roots(route_roots),
            allowed_public_entry_points,
            adapter_bridge_files,
            allowed_imports,
        })
    }

    fn violations(
        &self,
        project_root: &Path,
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        for edge in edges {
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            let source_location = self.system_location(project_root, &edge.source);
            if let Some(source_location) = &source_location {
                let source_area = self.system_area(source_location);
                if self.is_route_file(project_root, target)
                    && !self
                        .is_allowed_area_import(source_area, FeatureSystemDependencyArea::Routes)
                {
                    violations.push(Violation::feature_system_dependency_flow(
                        edge,
                        target,
                        source_area,
                        FeatureSystemDependencyArea::Routes,
                        source_location,
                        severity,
                        "feature systems should be imported by routes, not import route files"
                            .to_string(),
                    ));
                    continue;
                }
            }

            let Some(target_location) = self.system_location(project_root, target) else {
                continue;
            };
            let target_area = self.system_area(&target_location);
            if self.is_public_entry(&target_location) {
                continue;
            }

            match source_location {
                Some(source_location)
                    if source_location.system_path == target_location.system_path =>
                {
                    let source_area = self.system_area(&source_location);
                    if self.is_allowed_same_system_import(
                        source_area,
                        target_area,
                        &source_location,
                    ) {
                        continue;
                    }
                    violations.push(Violation::feature_system_dependency_flow(
                        edge,
                        target,
                        source_area,
                        target_area,
                        &target_location,
                        severity,
                        "route this dependency through an allowed lower area or update allowedImports if the flow is intentional".to_string(),
                    ));
                }
                Some(source_location) => {
                    violations.push(Violation::feature_system_dependency_flow(
                        edge,
                        target,
                        self.system_area(&source_location),
                        target_area,
                        &target_location,
                        severity,
                        format!(
                            "import from the {} public entry point instead",
                            target_location.system_path
                        ),
                    ));
                }
                None => {
                    let source_area = if self.is_route_file(project_root, &edge.source) {
                        FeatureSystemDependencyArea::Routes
                    } else {
                        FeatureSystemDependencyArea::Other
                    };
                    violations.push(Violation::feature_system_dependency_flow(
                        edge,
                        target,
                        source_area,
                        target_area,
                        &target_location,
                        severity,
                        format!(
                            "import from the {} public entry point instead",
                            target_location.system_path
                        ),
                    ));
                }
            }
        }

        violations
    }

    fn system_location(&self, project_root: &Path, file: &Path) -> Option<FeatureSystemLocation> {
        let components = project_relative_components(project_root, file);
        self.systems_roots.iter().find_map(|root| {
            if components.len() <= root.len() || !path_has_prefix_components(&components, root) {
                return None;
            }
            let domain = components[root.len()].clone();
            let system_components = &components[..=root.len()];
            let relative_components = &components[root.len() + 1..];
            Some(FeatureSystemLocation {
                domain,
                system_path: display_path_components(system_components),
                relative_file: display_path_components(relative_components),
            })
        })
    }

    fn system_area(&self, location: &FeatureSystemLocation) -> FeatureSystemDependencyArea {
        if self.is_public_entry(location) {
            return FeatureSystemDependencyArea::PublicEntry;
        }
        FeatureSystemDependencyArea::from_relative_file(&location.relative_file)
    }

    fn is_public_entry(&self, location: &FeatureSystemLocation) -> bool {
        self.allowed_public_entry_points
            .contains(&location.relative_file)
    }

    fn is_route_file(&self, project_root: &Path, file: &Path) -> bool {
        let components = project_relative_components(project_root, file);
        path_under_any_root(&components, &self.route_roots)
    }

    fn is_allowed_same_system_import(
        &self,
        source_area: FeatureSystemDependencyArea,
        target_area: FeatureSystemDependencyArea,
        source_location: &FeatureSystemLocation,
    ) -> bool {
        source_area == target_area
            || self.is_adapter_bridge(source_area, target_area, source_location)
            || self.is_allowed_area_import(source_area, target_area)
    }

    fn is_adapter_bridge(
        &self,
        source_area: FeatureSystemDependencyArea,
        target_area: FeatureSystemDependencyArea,
        source_location: &FeatureSystemLocation,
    ) -> bool {
        source_area == FeatureSystemDependencyArea::Lib
            && target_area == FeatureSystemDependencyArea::Adapters
            && self
                .adapter_bridge_files
                .contains(&source_location.relative_file)
    }

    fn is_allowed_area_import(
        &self,
        source_area: FeatureSystemDependencyArea,
        target_area: FeatureSystemDependencyArea,
    ) -> bool {
        self.allowed_imports
            .get(source_area.config_name())
            .is_some_and(|allowed| allowed.contains(target_area.config_name()))
    }
}

impl FeatureSystemAdapterContractPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let systems_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "systemsRoots",
            DEFAULT_SYSTEMS_ROOTS,
        )?;
        let route_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "routeRoots",
            DEFAULT_ROUTE_ROOTS,
        )?;
        let adapter_directory = string_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "adapterDirectory",
            DEFAULT_FEATURE_SYSTEM_ADAPTER_DIRECTORY,
        )?;
        let adapter_file_name_template = string_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "adapterFileNameTemplate",
            DEFAULT_FEATURE_SYSTEM_ADAPTER_FILE_TEMPLATE,
        )?;
        let api_export_name_template = string_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "apiExportNameTemplate",
            DEFAULT_FEATURE_SYSTEM_API_EXPORT_TEMPLATE,
        )?;
        let error_export_name_template = string_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "errorExportNameTemplate",
            DEFAULT_FEATURE_SYSTEM_API_ERROR_TEMPLATE,
        )?;
        let http_client_names = string_vec_option(
            RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
            setting,
            "httpClientNames",
            DEFAULT_FEATURE_SYSTEM_HTTP_CLIENT_NAMES,
        )?;

        Ok(Self {
            systems_roots: path_roots(systems_roots),
            route_roots: path_roots(route_roots),
            adapter_directory,
            adapter_file_name_template,
            api_export_name_template,
            error_export_name_template,
            http_client_names,
        })
    }

    fn violations(
        &self,
        project_root: &Path,
        files: &[PathBuf],
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let adapter_systems = self.adapter_systems(project_root, files);
        let file_set = files.iter().cloned().collect::<HashSet<_>>();

        for adapter_system in adapter_systems.values() {
            if !file_set.contains(&adapter_system.expected_adapter_file) {
                violations.push(Violation::feature_system_adapter_contract(
                    &adapter_system.representative_file,
                    severity,
                    format!(
                        "feature system adapter layer should expose {}",
                        adapter_system.expected_relative_file
                    ),
                    format!(
                        "add {} as the domain-named adapter file",
                        adapter_system.expected_relative_file
                    ),
                ));
                continue;
            }

            let source =
                fs::read_to_string(&adapter_system.expected_adapter_file).map_err(|source| {
                    OnionCryError::ReadSource {
                        path: adapter_system.expected_adapter_file.clone(),
                        source,
                    }
                })?;
            let expected_api_export =
                render_feature_template(&self.api_export_name_template, &adapter_system.domain);
            if !source_exports_value(&source, &expected_api_export) {
                violations.push(Violation::feature_system_adapter_contract(
                    &adapter_system.expected_adapter_file,
                    severity,
                    format!("adapter file should export namespace object {expected_api_export}"),
                    format!("export a const object named {expected_api_export}"),
                ));
            }

            let expected_error_export =
                render_feature_template(&self.error_export_name_template, &adapter_system.domain);
            if !source_exports_error_class(&source, &expected_error_export) {
                violations.push(Violation::feature_system_adapter_contract(
                    &adapter_system.expected_adapter_file,
                    severity,
                    format!("adapter file should export typed API error {expected_error_export}"),
                    format!("export class {expected_error_export} extends Error"),
                ));
            }

            if source_has_configured_read_call(&source, &self.http_client_names)
                && !source_accepts_and_passes_abort_signal(&source)
            {
                violations.push(Violation::feature_system_adapter_contract(
                    &adapter_system.expected_adapter_file,
                    severity,
                    "adapter read operations should accept and pass an AbortSignal".to_string(),
                    "accept a signal?: AbortSignal parameter and pass it to fetch or the configured HTTP client".to_string(),
                ));
            }
        }

        for edge in edges {
            let Some(source_location) = self.system_location(project_root, &edge.source) else {
                continue;
            };
            if !self.is_adapter_file(&source_location) {
                continue;
            }
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            if self.is_route_file(project_root, target) {
                violations.push(Violation::feature_system_adapter_contract_import(
                    edge,
                    target,
                    FeatureSystemDependencyArea::Routes,
                    severity,
                ));
                continue;
            }
            let Some(target_location) = self.system_location(project_root, target) else {
                continue;
            };
            let target_area =
                FeatureSystemDependencyArea::from_relative_file(&target_location.relative_file);
            if target_area.is_upper_frontend_area() {
                violations.push(Violation::feature_system_adapter_contract_import(
                    edge,
                    target,
                    target_area,
                    severity,
                ));
            }
        }

        Ok(violations)
    }

    fn adapter_systems(
        &self,
        project_root: &Path,
        files: &[PathBuf],
    ) -> BTreeMap<String, FeatureSystemAdapterInfo> {
        let mut systems = BTreeMap::new();

        for file in files {
            let Some(location) = self.system_location(project_root, file) else {
                continue;
            };
            if !self.is_adapter_file(&location) {
                continue;
            }
            let expected_file_name =
                render_feature_template(&self.adapter_file_name_template, &location.domain);
            let expected_relative_file =
                format!("{}/{}", self.adapter_directory, expected_file_name);
            let expected_adapter_file = normalize_path(
                &project_root
                    .join(&location.system_path)
                    .join(&expected_relative_file),
            );
            systems
                .entry(location.system_path.clone())
                .or_insert_with(|| FeatureSystemAdapterInfo {
                    domain: location.domain,
                    representative_file: file.clone(),
                    expected_adapter_file,
                    expected_relative_file,
                });
        }

        systems
    }

    fn system_location(&self, project_root: &Path, file: &Path) -> Option<FeatureSystemLocation> {
        let components = project_relative_components(project_root, file);
        self.systems_roots.iter().find_map(|root| {
            if components.len() <= root.len() || !path_has_prefix_components(&components, root) {
                return None;
            }
            let domain = components[root.len()].clone();
            let system_components = &components[..=root.len()];
            let relative_components = &components[root.len() + 1..];
            Some(FeatureSystemLocation {
                domain,
                system_path: display_path_components(system_components),
                relative_file: display_path_components(relative_components),
            })
        })
    }

    fn is_adapter_file(&self, location: &FeatureSystemLocation) -> bool {
        location
            .relative_file
            .split('/')
            .next()
            .is_some_and(|segment| segment == self.adapter_directory)
    }

    fn is_route_file(&self, project_root: &Path, file: &Path) -> bool {
        let components = project_relative_components(project_root, file);
        path_under_any_root(&components, &self.route_roots)
    }
}

impl FeatureSystemDependencyArea {
    pub(crate) fn from_relative_file(relative_file: &str) -> Self {
        match relative_file.split('/').next().unwrap_or_default() {
            "adapters" => Self::Adapters,
            "lib" => Self::Lib,
            "hooks" => Self::Hooks,
            "contexts" => Self::Contexts,
            "stores" => Self::Stores,
            "components" => Self::Components,
            "guards" => Self::Guards,
            _ => Self::Other,
        }
    }

    pub(crate) fn config_name(self) -> &'static str {
        match self {
            Self::PublicEntry => "public-entry",
            Self::Adapters => "adapters",
            Self::Lib => "lib",
            Self::Hooks => "hooks",
            Self::Contexts => "contexts",
            Self::Stores => "stores",
            Self::Components => "components",
            Self::Guards => "guards",
            Self::Routes => "routes",
            Self::Other => "other",
        }
    }

    pub(crate) fn display_name(self) -> &'static str {
        match self {
            Self::PublicEntry => "public entry",
            Self::Adapters => "adapters",
            Self::Lib => "lib",
            Self::Hooks => "hooks",
            Self::Contexts => "contexts",
            Self::Stores => "stores",
            Self::Components => "components",
            Self::Guards => "guards",
            Self::Routes => "routes",
            Self::Other => "outside code",
        }
    }
}

impl FeatureSystemDependencyArea {
    fn is_upper_frontend_area(self) -> bool {
        matches!(
            self,
            Self::Hooks | Self::Contexts | Self::Stores | Self::Components | Self::Guards
        )
    }
}

impl FeatureSystemQueryContractPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let systems_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
            setting,
            "systemsRoots",
            DEFAULT_SYSTEMS_ROOTS,
        )?;
        let route_roots = string_vec_option(
            RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
            setting,
            "routeRoots",
            DEFAULT_ROUTE_ROOTS,
        )?;
        let adapter_directory = string_option(
            RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
            setting,
            "adapterDirectory",
            DEFAULT_FEATURE_SYSTEM_ADAPTER_DIRECTORY,
        )?;
        let query_keys_file = string_option(
            RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
            setting,
            "queryKeysFile",
            DEFAULT_FEATURE_SYSTEM_QUERY_KEYS_FILE,
        )?;
        let query_options_file = string_option(
            RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
            setting,
            "queryOptionsFile",
            DEFAULT_FEATURE_SYSTEM_QUERY_OPTIONS_FILE,
        )?;

        Ok(Self {
            systems_roots: path_roots(systems_roots),
            route_roots: path_roots(route_roots),
            adapter_directory,
            query_keys_file,
            query_options_file,
        })
    }

    fn violations(
        &self,
        project_root: &Path,
        files: &[PathBuf],
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Result<Vec<Violation>> {
        let source_by_file = read_source_files(files)?;
        let mut query_states = self.query_states(project_root, files, &source_by_file);
        self.mark_adapter_backed_reads(project_root, edges, &source_by_file, &mut query_states);
        self.mark_route_owned_queries(project_root, edges, &source_by_file, &mut query_states);

        let file_set = files.iter().cloned().collect::<HashSet<_>>();
        let mut violations = Vec::new();

        for state in query_states.values() {
            let query_keys_path =
                self.expected_system_file(project_root, state, &self.query_keys_file);
            if state.requires_query_keys && !file_set.contains(&query_keys_path) {
                violations.push(Violation::feature_system_query_contract(
                    &state.representative_file,
                    severity,
                    format!("{} requires {}", state.domain, self.query_keys_file),
                    format!("add {} under {}", self.query_keys_file, state.system_path),
                ));
            }

            let query_options_path =
                self.expected_system_file(project_root, state, &self.query_options_file);
            if state.requires_query_options && !file_set.contains(&query_options_path) {
                violations.push(Violation::feature_system_query_contract(
                    &state.representative_file,
                    severity,
                    format!("{} requires {}", state.domain, self.query_options_file),
                    format!(
                        "add {} under {}",
                        self.query_options_file, state.system_path
                    ),
                ));
            }

            if let Some(query_options_source) = source_by_file.get(&query_options_path) {
                violations.extend(self.query_options_violations(
                    &query_options_path,
                    query_options_source,
                    project_root,
                    edges,
                    severity,
                ));
            }

            for file in &state.files {
                let Some(source) = source_by_file.get(file) else {
                    continue;
                };
                let Some(location) = self.system_location(project_root, file) else {
                    continue;
                };
                let area = FeatureSystemDependencyArea::from_relative_file(&location.relative_file);
                if area == FeatureSystemDependencyArea::Hooks {
                    violations.extend(self.hook_violations(
                        file,
                        source,
                        project_root,
                        edges,
                        severity,
                    ));
                }
                if area == FeatureSystemDependencyArea::Components
                    && source_declares_query_key(source)
                {
                    violations.push(Violation::feature_system_query_contract(
                        file,
                        severity,
                        "components should not own query keys".to_string(),
                        format!(
                            "move the query key to {} and reuse a query option factory",
                            self.query_keys_file
                        ),
                    ));
                }
            }
        }

        for file in files {
            if !self.is_route_file(project_root, file) {
                continue;
            }
            let Some(source) = source_by_file.get(file) else {
                continue;
            };
            if source_declares_query_key(source) {
                violations.push(Violation::feature_system_query_contract(
                    file,
                    severity,
                    "routes should not own query keys".to_string(),
                    format!(
                        "move the query key to a feature system {} file",
                        self.query_keys_file
                    ),
                ));
            }
        }

        Ok(violations)
    }

    fn query_states(
        &self,
        project_root: &Path,
        files: &[PathBuf],
        source_by_file: &BTreeMap<PathBuf, String>,
    ) -> BTreeMap<String, FeatureSystemQueryState> {
        let mut query_states = BTreeMap::<String, FeatureSystemQueryState>::new();

        for file in files {
            let Some(location) = self.system_location(project_root, file) else {
                continue;
            };
            let source = source_by_file.get(file).map_or("", String::as_str);
            let state = query_states
                .entry(location.system_path.clone())
                .or_insert_with(|| FeatureSystemQueryState {
                    domain: location.domain.clone(),
                    system_path: location.system_path.clone(),
                    representative_file: file.clone(),
                    files: Vec::new(),
                    requires_query_keys: false,
                    requires_query_options: false,
                });
            state.files.push(file.clone());
            if source_has_query_ownership(source) {
                state.requires_query_keys = true;
            }
            if source_uses_query_options_surface(source) {
                state.requires_query_options = true;
            }
        }

        query_states
    }

    fn mark_adapter_backed_reads(
        &self,
        project_root: &Path,
        edges: &[ImportEdge],
        source_by_file: &BTreeMap<PathBuf, String>,
        query_states: &mut BTreeMap<String, FeatureSystemQueryState>,
    ) {
        for edge in edges {
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            let Some(source_location) = self.system_location(project_root, &edge.source) else {
                continue;
            };
            let Some(target_location) = self.system_location(project_root, target) else {
                continue;
            };
            if source_location.system_path != target_location.system_path
                || !self.is_adapter_file(&target_location)
            {
                continue;
            }
            let source = source_by_file.get(&edge.source).map_or("", String::as_str);
            if !source_has_query_ownership(source)
                && source_location.relative_file != self.query_options_file
            {
                continue;
            }
            if let Some(state) = query_states.get_mut(&source_location.system_path) {
                state.requires_query_keys = true;
                state.requires_query_options = true;
            }
        }
    }

    fn mark_route_owned_queries(
        &self,
        project_root: &Path,
        edges: &[ImportEdge],
        source_by_file: &BTreeMap<PathBuf, String>,
        query_states: &mut BTreeMap<String, FeatureSystemQueryState>,
    ) {
        for edge in edges {
            if !self.is_route_file(project_root, &edge.source) {
                continue;
            }
            let source = source_by_file.get(&edge.source).map_or("", String::as_str);
            if !source_uses_query_options_surface(source) {
                continue;
            }
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            let Some(target_location) = self.system_location(project_root, target) else {
                continue;
            };
            if let Some(state) = query_states.get_mut(&target_location.system_path) {
                state.requires_query_keys = true;
                state.requires_query_options = true;
            }
        }
    }

    fn query_options_violations(
        &self,
        file: &Path,
        source: &str,
        project_root: &Path,
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        if !source_imports_and_uses_query_options(source) {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "query option files should import and use queryOptions from @tanstack/react-query"
                    .to_string(),
                "import queryOptions from @tanstack/react-query and wrap option factories with queryOptions".to_string(),
            ));
        }
        if source.contains("queryOptions(")
            && !(source.contains("queryKey") && source.contains("queryFn"))
        {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "query option factories should co-locate queryKey and queryFn".to_string(),
                "define queryKey and queryFn in the same queryOptions factory".to_string(),
            ));
        }
        if self.imports_adapter(project_root, file, edges) && !source_passes_query_signal(source) {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "query functions should pass the query context signal to adapters".to_string(),
                "destructure signal in queryFn and pass it to the adapter call".to_string(),
            ));
        }

        violations
    }

    fn hook_violations(
        &self,
        file: &Path,
        source: &str,
        project_root: &Path,
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        if source_has_query_hook_read(source)
            && (source_declares_query_key(source)
                || source.contains("queryFn")
                || !self.imports_query_options(project_root, file, edges))
        {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "query hooks should reuse factories from lib/query-options.ts".to_string(),
                format!(
                    "import a factory from {} instead of declaring queryKey or queryFn inline",
                    self.query_options_file
                ),
            ));
        }

        if source.contains("useMutation(") && !source_has_mutation_invalidation(source) {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "mutation hooks should invalidate relevant queries in onSuccess or onSettled"
                    .to_string(),
                "add an onSuccess or onSettled handler that calls invalidateQueries".to_string(),
            ));
        }

        if source.contains("onMutate") && !source_has_optimistic_update_contract(source) {
            violations.push(Violation::feature_system_query_contract(
                file,
                severity,
                "optimistic cache updates should cancel, snapshot or rollback, and invalidate on settlement".to_string(),
                "include cancelQueries, a previous data snapshot or rollback, and settlement invalidation".to_string(),
            ));
        }

        violations
    }

    fn imports_adapter(&self, project_root: &Path, file: &Path, edges: &[ImportEdge]) -> bool {
        edges.iter().any(|edge| {
            edge.source == file
                && matches!(&edge.resolution, ImportResolution::Local(target) if self
                    .system_location(project_root, target)
                    .is_some_and(|location| self.is_adapter_file(&location)))
        })
    }

    fn imports_query_options(
        &self,
        project_root: &Path,
        file: &Path,
        edges: &[ImportEdge],
    ) -> bool {
        edges.iter().any(|edge| {
            edge.source == file
                && matches!(&edge.resolution, ImportResolution::Local(target) if self
                    .system_location(project_root, target)
                    .is_some_and(|location| location.relative_file == self.query_options_file))
        })
    }

    fn expected_system_file(
        &self,
        project_root: &Path,
        state: &FeatureSystemQueryState,
        relative_file: &str,
    ) -> PathBuf {
        normalize_path(&project_root.join(&state.system_path).join(relative_file))
    }

    fn system_location(&self, project_root: &Path, file: &Path) -> Option<FeatureSystemLocation> {
        let components = project_relative_components(project_root, file);
        self.systems_roots.iter().find_map(|root| {
            if components.len() <= root.len() || !path_has_prefix_components(&components, root) {
                return None;
            }
            let domain = components[root.len()].clone();
            let system_components = &components[..=root.len()];
            let relative_components = &components[root.len() + 1..];
            Some(FeatureSystemLocation {
                domain,
                system_path: display_path_components(system_components),
                relative_file: display_path_components(relative_components),
            })
        })
    }

    fn is_adapter_file(&self, location: &FeatureSystemLocation) -> bool {
        location
            .relative_file
            .split('/')
            .next()
            .is_some_and(|segment| segment == self.adapter_directory)
    }

    fn is_route_file(&self, project_root: &Path, file: &Path) -> bool {
        let components = project_relative_components(project_root, file);
        path_under_any_root(&components, &self.route_roots)
    }
}

fn render_feature_template(template: &str, domain: &str) -> String {
    template
        .replace("{domain}", domain)
        .replace("{domainCamel}", &domain_camel_case(domain))
        .replace("{DomainPascal}", &domain_pascal_case(domain))
}

fn domain_camel_case(domain: &str) -> String {
    let pascal = domain_pascal_case(domain);
    let mut characters = pascal.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_ascii_lowercase(),
        characters.collect::<String>()
    )
}

fn domain_pascal_case(domain: &str) -> String {
    domain
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            let Some(first) = characters.next() else {
                return String::new();
            };
            format!("{}{}", first.to_ascii_uppercase(), characters.as_str())
        })
        .collect()
}

fn source_exports_value(source: &str, name: &str) -> bool {
    source.contains(&format!("export const {name}"))
        || source.contains(&format!("export let {name}"))
        || source.contains(&format!("export var {name}"))
        || (source.contains(&format!("const {name}"))
            && source.contains(&format!("export {{ {name} }}")))
}

fn source_exports_error_class(source: &str, name: &str) -> bool {
    (source.contains(&format!("export class {name}")) && source.contains("extends Error"))
        || (source.contains(&format!("class {name}"))
            && source.contains("extends Error")
            && source.contains(&format!("export {{ {name} }}")))
}

fn source_has_configured_read_call(source: &str, http_client_names: &[String]) -> bool {
    http_client_names.iter().any(|client| {
        if client == "fetch" {
            source.contains("fetch(")
        } else {
            source.contains(&format!("{client}("))
        }
    })
}

fn source_accepts_and_passes_abort_signal(source: &str) -> bool {
    source.contains("AbortSignal") && source.matches("signal").count() >= 2
}

fn read_source_files(files: &[PathBuf]) -> Result<BTreeMap<PathBuf, String>> {
    let mut sources = BTreeMap::new();
    for file in files {
        if !is_source_file(file) {
            continue;
        }
        let source = fs::read_to_string(file).map_err(|source| OnionCryError::ReadSource {
            path: file.clone(),
            source,
        })?;
        sources.insert(file.clone(), source);
    }
    Ok(sources)
}

fn source_has_query_ownership(source: &str) -> bool {
    source_has_query_hook_read(source)
        || source.contains("prefetchQuery(")
        || source.contains("fetchQuery(")
        || source.contains("ensureQueryData(")
        || source.contains("getQueryData(")
}

fn source_uses_query_options_surface(source: &str) -> bool {
    source_has_query_hook_read(source)
        || source.contains("prefetchQuery(")
        || source.contains("fetchQuery(")
        || source.contains("ensureQueryData(")
        || source.contains("getQueryData(")
}

fn source_has_query_hook_read(source: &str) -> bool {
    source.contains("useQuery(")
        || source.contains("useSuspenseQuery(")
        || source.contains("useInfiniteQuery(")
}

fn source_imports_and_uses_query_options(source: &str) -> bool {
    source.contains("@tanstack/react-query")
        && source.contains("queryOptions")
        && source.contains("queryOptions(")
}

fn source_declares_query_key(source: &str) -> bool {
    source.contains("queryKey")
}

fn source_passes_query_signal(source: &str) -> bool {
    source.contains("queryFn") && source.matches("signal").count() >= 2
}

fn source_has_mutation_invalidation(source: &str) -> bool {
    source.contains("useMutation(")
        && (source.contains("onSuccess") || source.contains("onSettled"))
        && source.contains("invalidateQueries")
}

fn source_has_optimistic_update_contract(source: &str) -> bool {
    source.contains("cancelQueries")
        && (source.contains("previous")
            || source.contains("snapshot")
            || source.contains("rollback"))
        && (source.contains("onSettled") || source.contains("invalidateQueries"))
}
