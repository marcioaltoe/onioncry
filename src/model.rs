use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailOn {
    Error,
    Warning,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub status: CheckStatus,
    pub summary: CheckSummary,
    pub violations: Vec<Violation>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplainReport {
    pub file: String,
    pub layer: BoundaryExplanation,
    pub context: BoundaryExplanation,
    pub public_surface: bool,
    pub imports: Vec<ImportExplanation>,
    pub violations: Vec<Violation>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundaryExplanation {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub matched_patterns: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportExplanation {
    pub specifier: String,
    pub kind: String,
    pub type_only: bool,
    pub line: usize,
    pub column: usize,
    pub resolution: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_allowed: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Fail,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckSummary {
    pub file_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub violation_count: usize,
    #[serde(skip_serializing_if = "is_zero")]
    pub baselined_count: usize,
    #[serde(skip_serializing_if = "is_zero")]
    pub suppressed_count: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Violation {
    pub rule: String,
    pub severity: String,
    pub message: String,
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_specifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_layer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_layer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_path: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_layers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_contexts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "is_false")]
    pub baselined: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub suppressed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppression_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportEdge {
    pub source: PathBuf,
    pub specifier: String,
    pub kind: ImportKind,
    pub type_only: bool,
    pub line: usize,
    pub column: usize,
    pub resolution: ImportResolution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportKind {
    StaticImport,
    ReExport,
    DynamicImport,
    Require,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportResolution {
    Local(PathBuf),
    External,
    UnresolvedLocal,
}

impl CheckReport {
    pub fn should_exit_with_failure(&self) -> bool {
        matches!(self.status, CheckStatus::Fail)
    }
}

impl CheckStatus {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            CheckStatus::Pass => "pass",
            CheckStatus::Fail => "fail",
        }
    }
}

impl ImportKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ImportKind::StaticImport => "staticImport",
            ImportKind::ReExport => "reExport",
            ImportKind::DynamicImport => "dynamicImport",
            ImportKind::Require => "require",
        }
    }
}

fn is_false(value: &bool) -> bool {
    !value
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}
