use super::location::{is_public_entry, is_route_file, system_location};
use crate::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) struct FeatureSystemPublicApiPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    allowed_public_entry_points: HashSet<String>,
    reject_wildcard_reexports: bool,
}

impl FeatureSystemPublicApiPolicy {
    pub(super) fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
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

    pub(super) fn violations(
        &self,
        project_root: &Path,
        files: &[PathBuf],
        edges: &[ImportEdge],
        severity: Severity,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        if self.reject_wildcard_reexports {
            for file in files {
                let Some(location) = system_location(project_root, file, &self.systems_roots)
                else {
                    continue;
                };
                if !is_public_entry(&location, &self.allowed_public_entry_points) {
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
            let Some(target_location) = system_location(project_root, target, &self.systems_roots)
            else {
                continue;
            };
            if is_public_entry(&target_location, &self.allowed_public_entry_points) {
                continue;
            }
            if system_location(project_root, &edge.source, &self.systems_roots).is_some_and(
                |source_location| source_location.system_path == target_location.system_path,
            ) {
                continue;
            }

            violations.push(Violation::feature_system_public_api_internal_import(
                edge,
                target,
                &target_location,
                is_route_file(project_root, &edge.source, &self.route_roots),
                severity,
            ));
        }

        Ok(violations)
    }
}
