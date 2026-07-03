use super::helpers::{
    read_source_files, source_declares_query_key, source_has_mutation_invalidation,
    source_has_optimistic_update_contract, source_has_query_hook_read, source_has_query_ownership,
    source_imports_and_uses_query_options, source_passes_query_signal,
    source_uses_query_options_surface,
};
use super::location::{is_file_in_area, is_route_file, system_location};
use super::{FeatureSystemDependencyArea, FeatureSystemLocation};
use crate::*;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) struct FeatureSystemQueryContractPolicy {
    systems_roots: Vec<Vec<String>>,
    route_roots: Vec<Vec<String>>,
    adapter_directory: String,
    query_keys_file: String,
    query_options_file: String,
}

struct FeatureSystemQueryState {
    domain: String,
    system_path: String,
    representative_file: PathBuf,
    files: Vec<PathBuf>,
    requires_query_keys: bool,
    requires_query_options: bool,
}

impl FeatureSystemQueryContractPolicy {
    pub(super) fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
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

    pub(super) fn violations(
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
                let Some(location) = system_location(project_root, file, &self.systems_roots)
                else {
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
            if !is_route_file(project_root, file, &self.route_roots) {
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
            let Some(location) = system_location(project_root, file, &self.systems_roots) else {
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
            let Some(source_location) =
                system_location(project_root, &edge.source, &self.systems_roots)
            else {
                continue;
            };
            let Some(target_location) = system_location(project_root, target, &self.systems_roots)
            else {
                continue;
            };
            if source_location.system_path != target_location.system_path
                || !is_file_in_area(&target_location, &self.adapter_directory)
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
            if !is_route_file(project_root, &edge.source, &self.route_roots) {
                continue;
            }
            let source = source_by_file.get(&edge.source).map_or("", String::as_str);
            if !source_uses_query_options_surface(source) {
                continue;
            }
            let ImportResolution::Local(target) = &edge.resolution else {
                continue;
            };
            let Some(target_location) = system_location(project_root, target, &self.systems_roots)
            else {
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
        self.imports_matching_system_location(project_root, file, edges, |location| {
            is_file_in_area(location, &self.adapter_directory)
        })
    }

    fn imports_query_options(
        &self,
        project_root: &Path,
        file: &Path,
        edges: &[ImportEdge],
    ) -> bool {
        self.imports_matching_system_location(project_root, file, edges, |location| {
            location.relative_file == self.query_options_file
        })
    }

    fn imports_matching_system_location(
        &self,
        project_root: &Path,
        file: &Path,
        edges: &[ImportEdge],
        matches_location: impl Fn(&FeatureSystemLocation) -> bool,
    ) -> bool {
        edges.iter().any(|edge| {
            if edge.source != file {
                return false;
            }
            let ImportResolution::Local(target) = &edge.resolution else {
                return false;
            };

            system_location(project_root, target, &self.systems_roots)
                .is_some_and(|location| matches_location(&location))
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
}
