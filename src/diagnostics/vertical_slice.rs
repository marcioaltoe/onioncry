use crate::rules::catalog::{
    RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT, RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS,
    RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS, RULE_VERTICAL_SLICE_ENTRY_POINT,
};
use crate::{ImportEdge, Severity, SliceLocation, Violation};
use std::path::Path;

impl Violation {
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
            baselined: false,
            suppressed: false,
            suppression_reason: None,
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
            baselined: false,
            suppressed: false,
            suppression_reason: None,
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
            baselined: false,
            suppressed: false,
            suppression_reason: None,
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
            baselined: false,
            suppressed: false,
            suppression_reason: None,
        }
    }
}
