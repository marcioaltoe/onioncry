use super::location::{is_public_entry, is_route_file, system_location};
use super::{FeatureSystemDependencyArea, FeatureSystemLocation};
use crate::rules::catalog::RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW;
use crate::{
    DEFAULT_FEATURE_SYSTEM_ADAPTER_BRIDGE_FILES, DEFAULT_FEATURE_SYSTEM_ALLOWED_IMPORTS,
    DEFAULT_FEATURE_SYSTEM_PUBLIC_ENTRY_POINTS, DEFAULT_ROUTE_ROOTS, DEFAULT_SYSTEMS_ROOTS,
    ImportEdge, ImportResolution, Result, RuleSetting, Severity, Violation, path_roots,
    string_set_map_option, string_set_option, string_vec_option,
};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

pub(super) struct FeatureSystemDependencyFlowPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    allowed_public_entry_points: HashSet<String>,
    adapter_bridge_files: HashSet<String>,
    allowed_imports: BTreeMap<String, HashSet<String>>,
}

impl FeatureSystemDependencyFlowPolicy {
    pub(super) fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
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

    pub(super) fn violations(
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
            let source_location = system_location(project_root, &edge.source, &self.systems_roots);
            if let Some(source_location) = &source_location {
                let source_area = self.system_area(source_location);
                if is_route_file(project_root, target, &self.route_roots)
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

            let Some(target_location) = system_location(project_root, target, &self.systems_roots)
            else {
                continue;
            };
            let target_area = self.system_area(&target_location);
            if is_public_entry(&target_location, &self.allowed_public_entry_points) {
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
                    let source_area =
                        if is_route_file(project_root, &edge.source, &self.route_roots) {
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

    fn system_area(&self, location: &FeatureSystemLocation) -> FeatureSystemDependencyArea {
        if is_public_entry(location, &self.allowed_public_entry_points) {
            return FeatureSystemDependencyArea::PublicEntry;
        }
        FeatureSystemDependencyArea::from_relative_file(&location.relative_file)
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
