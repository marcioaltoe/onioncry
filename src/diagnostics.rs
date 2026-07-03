use crate::*;
use std::collections::HashSet;
use std::path::Path;

impl Violation {
    pub(crate) fn unclassified_file(file: &Path, severity: Severity) -> Self {
        Self {
            rule: RULE_UNCLASSIFIED_FILE.to_string(),
            severity: severity.as_str().to_string(),
            message: "file is not classified by any configured architectural layer".to_string(),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "add a matching layers.*.patterns entry or exclude the file".to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn ambiguous_layer(
        file: &Path,
        matched_layers: Vec<String>,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_AMBIGUOUS_LAYER.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "file matches multiple architectural layers: {}",
                matched_layers.join(", ")
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some("make layer patterns mutually exclusive".to_string()),
            matched_layers: Some(matched_layers),
            matched_contexts: None,
        }
    }

    pub(crate) fn layer_leak(
        edge: &ImportEdge,
        target: &Path,
        from_layer: &str,
        to_layer: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_LAYER_LEAK.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not import {to_layer} through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: Some(to_layer.to_string()),
            from_context: None,
            to_context: None,
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(format!(
                "add {to_layer:?} to layers.{from_layer}.mayImport only if this dependency is intentional"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn forbidden_external_package(
        edge: &ImportEdge,
        from_layer: &str,
        package_name: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not import external package {package_name} through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: Some(package_name.to_string()),
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "add {package_name:?} to the cleanarch/no-forbidden-imports allowlist for {from_layer} only if this package is domain-safe for that layer"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn framework_in_core(
        edge: &ImportEdge,
        from_layer: &str,
        package_name: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_FRAMEWORK_IN_CORE.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not depend on framework package {package_name} through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: Some(package_name.to_string()),
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "depend on a core-owned port and move framework code to an outer layer".to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn outer_data_format_in_core(
        edge: &ImportEdge,
        target: &Path,
        from_layer: &str,
        to_layer: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_OUTER_DATA_FORMAT_IN_CORE.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not import {to_layer} data format through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: Some(to_layer.to_string()),
            from_context: None,
            to_context: None,
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "define a core-owned type or mapper instead of importing outer data formats"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn ambiguous_context(
        file: &Path,
        matched_contexts: Vec<String>,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_AMBIGUOUS_CONTEXT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "file matches multiple architectural contexts: {}",
                matched_contexts.join(", ")
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some("make context patterns mutually exclusive".to_string()),
            matched_layers: None,
            matched_contexts: Some(matched_contexts),
        }
    }

    pub(crate) fn cross_context_internal_import(
        edge: &ImportEdge,
        target: &Path,
        from_context: &str,
        to_context: &str,
        severity: Severity,
        public_surface_segments: &HashSet<String>,
    ) -> Self {
        let mut segments = public_surface_segments.iter().cloned().collect::<Vec<_>>();
        segments.sort();
        Self {
            rule: RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_context} may not import {to_context} internal details through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: Some(from_context.to_string()),
            to_context: Some(to_context.to_string()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(format!(
                "import from the {to_context} public surface segment instead{}",
                if segments.is_empty() {
                    String::new()
                } else {
                    format!(": {}", segments.join(", "))
                }
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn public_surface_internal_reexport(
        edge: &ImportEdge,
        target: &Path,
        context: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{context} public surface may not re-export internal detail through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: Some(context.to_string()),
            to_context: Some(context.to_string()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "move the contract into the public surface or stop re-exporting it".to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn context_cycle(
        edge: &ImportEdge,
        target: &Path,
        context_path: &[String],
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_CONTEXT_CYCLE.to_string(),
            severity: severity.as_str().to_string(),
            message: format!("context dependency cycle: {}", context_path.join(" -> ")),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: context_path.first().cloned(),
            to_context: context_path.get(1).cloned(),
            target_file: Some(target.display().to_string()),
            cycle_path: Some(context_path.to_vec()),
            suggestion: Some(
                "extract a public contract or shared kernel so context dependencies point one way"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn unowned_schema_import(
        edge: &ImportEdge,
        target: &Path,
        from_context: &str,
        to_context: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_UNOWNED_SCHEMA_IMPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_context} may not import {to_context} owned schema through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: Some(from_context.to_string()),
            to_context: Some(to_context.to_string()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "depend on the owning context contract instead of importing its storage schema"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn concrete_dependency(
        edge: &ImportEdge,
        target: Option<&Path>,
        from_layer: &str,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_NO_CONCRETE_DEPENDENCY.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_layer} may not depend on concrete detail through {}",
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(from_layer.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: target.map(|target| target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "depend on an abstraction owned by the core layer and bind the concrete detail outside"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn feature_envy(
        file: &Path,
        from_context: &str,
        to_context: &str,
        import_count: usize,
        own_context_count: usize,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_ENVY.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{from_context} file imports {import_count} dependencies from {to_context} and {own_context_count} from its own context"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: Some(from_context.to_string()),
            to_context: Some(to_context.to_string()),
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "move the behavior closer to the context it uses or depend on a smaller public contract"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn clean_artifact_placement(
        file: &Path,
        severity: Severity,
        role: &str,
        expected_layer: &str,
        expected_boundary: &str,
    ) -> Self {
        Self {
            rule: RULE_CLEAN_ARTIFACT_PLACEMENT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "cleanArchitecture artifact {role:?} should live in the {expected_layer} boundary {expected_boundary}"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: Some(expected_layer.to_string()),
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "move this {role} artifact to {expected_boundary} or turn cleanarch/artifact-placement off with an override while migrating"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn cross_slice_internal_import(
        edge: &ImportEdge,
        target: &Path,
        severity: Severity,
        source_location: &SliceLocation,
        target_location: &SliceLocation,
    ) -> Self {
        Self {
            rule: RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "verticalSlice slice {} may not import {} internals through {}",
                source_location.slice, target_location.slice, edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: Some(source_location.slice.clone()),
            to_context: Some(target_location.slice.clone()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(format!(
                "import from the {} public surface instead, such as index.ts or contracts/",
                target_location.slice_path
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn global_slice_artifact(
        file: &Path,
        severity: Severity,
        role: &str,
        slice_root_pattern: &str,
    ) -> Self {
        Self {
            rule: RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "verticalSlice artifact {role:?} is outside the configured slice layout {slice_root_pattern}"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: Some(role.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "move this slice artifact under {slice_root_pattern} or add its global folder to architecture.verticalSlice.allowedGlobalFolders"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn vertical_slice_entry_point(
        file: &Path,
        severity: Severity,
        slice: &str,
        slice_path: &str,
        entry_point_names: &str,
    ) -> Self {
        Self {
            rule: RULE_VERTICAL_SLICE_ENTRY_POINT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "verticalSlice slice {slice} does not expose a configured entry point"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: Some(slice.to_string()),
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "export one entry point from {slice_path}, such as one of {entry_point_names}"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn vertical_shared_layer_artifact(
        file: &Path,
        severity: Severity,
        folder: &str,
        slice_root_pattern: &str,
    ) -> Self {
        Self {
            rule: RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "verticalSlice shared layer folder {folder:?} is outside the configured slice layout"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: Some(folder.to_string()),
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(format!(
                "move this artifact under {slice_root_pattern} or keep only cross-cutting platform code in configured global folders"
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn shotgun_surgery(
        file: &Path,
        commit_count: usize,
        related_file_count: usize,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_SHOTGUN_SURGERY.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "file changed in {commit_count} commits with {related_file_count} recurring companion files"
            ),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "look for scattered responsibilities and extract a boundary that changes together"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn misplaced_test_file(file: &Path, severity: Severity, suggestion: String) -> Self {
        Self {
            rule: RULE_TEST_PLACEMENT.to_string(),
            severity: severity.as_str().to_string(),
            message: "test file is not in an allowed test location".to_string(),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn path_naming(
        file: &Path,
        severity: Severity,
        message: String,
        suggestion: String,
    ) -> Self {
        Self {
            rule: RULE_PATH_NAMING.to_string(),
            severity: severity.as_str().to_string(),
            message,
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn feature_system_layout(
        file: &Path,
        severity: Severity,
        message: String,
        suggestion: String,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_LAYOUT.to_string(),
            severity: severity.as_str().to_string(),
            message,
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn feature_system_public_api_wildcard(file: &Path, severity: Severity) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_PUBLIC_API.to_string(),
            severity: severity.as_str().to_string(),
            message: "feature system public API must not use wildcard re-exports".to_string(),
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(
                "replace wildcard re-exports with explicit named exports from the system barrel"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn feature_system_public_api_internal_import(
        edge: &ImportEdge,
        target: &Path,
        target_location: &FeatureSystemLocation,
        source_is_route: bool,
        severity: Severity,
    ) -> Self {
        let source_kind = if source_is_route {
            "route"
        } else {
            "outside code"
        };
        Self {
            rule: RULE_FEATURE_SYSTEM_PUBLIC_API.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{source_kind} may not import {} system internal file through {}",
                target_location.domain, edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: Some(target_location.domain.clone()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(format!(
                "import from the {} public entry point instead",
                target_location.system_path
            )),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn feature_system_dependency_flow(
        edge: &ImportEdge,
        target: &Path,
        source_area: FeatureSystemDependencyArea,
        target_area: FeatureSystemDependencyArea,
        target_location: &FeatureSystemLocation,
        severity: Severity,
        suggestion: String,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "{} may not import {} in the {} system through {}",
                source_area.display_name(),
                target_area.display_name(),
                target_location.domain,
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some(source_area.config_name().to_string()),
            to_layer: Some(target_area.config_name().to_string()),
            from_context: None,
            to_context: Some(target_location.domain.clone()),
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn feature_system_adapter_contract(
        file: &Path,
        severity: Severity,
        message: String,
        suggestion: String,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT.to_string(),
            severity: severity.as_str().to_string(),
            message,
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn feature_system_adapter_contract_import(
        edge: &ImportEdge,
        target: &Path,
        target_area: FeatureSystemDependencyArea,
        severity: Severity,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT.to_string(),
            severity: severity.as_str().to_string(),
            message: format!(
                "adapter files may not import {} through {}",
                target_area.display_name(),
                edge.specifier
            ),
            file: edge.source.display().to_string(),
            import_specifier: Some(edge.specifier.clone()),
            package_name: None,
            line: Some(edge.line),
            column: Some(edge.column),
            from_layer: Some("adapters".to_string()),
            to_layer: Some(target_area.config_name().to_string()),
            from_context: None,
            to_context: None,
            target_file: Some(target.display().to_string()),
            cycle_path: None,
            suggestion: Some(
                "move the dependency behind lib/query-options, a store, or another upper layer"
                    .to_string(),
            ),
            matched_layers: None,
            matched_contexts: None,
        }
    }

    pub(crate) fn feature_system_query_contract(
        file: &Path,
        severity: Severity,
        message: String,
        suggestion: String,
    ) -> Self {
        Self {
            rule: RULE_FEATURE_SYSTEM_QUERY_CONTRACT.to_string(),
            severity: severity.as_str().to_string(),
            message,
            file: file.display().to_string(),
            import_specifier: None,
            package_name: None,
            line: None,
            column: None,
            from_layer: None,
            to_layer: None,
            from_context: None,
            to_context: None,
            target_file: None,
            cycle_path: None,
            suggestion: Some(suggestion),
            matched_layers: None,
            matched_contexts: None,
        }
    }
}
