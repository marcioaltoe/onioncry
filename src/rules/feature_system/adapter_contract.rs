use super::helpers::{
    render_feature_template, source_accepts_and_passes_abort_signal, source_exports_error_class,
    source_exports_value, source_has_configured_read_call,
};
use super::location::{is_file_in_area, is_route_file, system_location};
use super::{FeatureSystemDependencyArea, FeatureSystemLocation};
use crate::rules::catalog::RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT;
use crate::{
    DEFAULT_FEATURE_SYSTEM_ADAPTER_DIRECTORY, DEFAULT_FEATURE_SYSTEM_ADAPTER_FILE_TEMPLATE,
    DEFAULT_FEATURE_SYSTEM_API_ERROR_TEMPLATE, DEFAULT_FEATURE_SYSTEM_API_EXPORT_TEMPLATE,
    DEFAULT_FEATURE_SYSTEM_HTTP_CLIENT_NAMES, DEFAULT_ROUTE_ROOTS, DEFAULT_SYSTEMS_ROOTS,
    ImportEdge, ImportResolution, OnionCryError, Result, RuleSetting, Severity, Violation,
    normalize_path, path_roots, string_option, string_vec_option,
};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) struct FeatureSystemAdapterContractPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    adapter_directory: String,
    adapter_file_name_template: String,
    api_export_name_template: String,
    error_export_name_template: String,
    http_client_names: Vec<String>,
}

struct FeatureSystemAdapterInfo {
    domain: String,
    representative_file: PathBuf,
    expected_adapter_file: PathBuf,
    expected_relative_file: String,
}

impl FeatureSystemAdapterContractPolicy {
    pub(super) fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
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

    pub(super) fn violations(
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
            let Some(source_location) =
                system_location(project_root, &edge.source, &self.systems_roots)
            else {
                continue;
            };
            if !self.is_adapter_file(&source_location) {
                continue;
            }
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            if is_route_file(project_root, target, &self.route_roots) {
                violations.push(Violation::feature_system_adapter_contract_import(
                    edge,
                    target,
                    FeatureSystemDependencyArea::Routes,
                    severity,
                ));
                continue;
            }
            let Some(target_location) = system_location(project_root, target, &self.systems_roots)
            else {
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
            let Some(location) = system_location(project_root, file, &self.systems_roots) else {
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

    fn is_adapter_file(&self, location: &FeatureSystemLocation) -> bool {
        is_file_in_area(location, &self.adapter_directory)
    }
}
