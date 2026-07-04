use super::builder::base_violation;
use crate::rules::catalog::{
    RULE_INVALID_SUPPRESSION, RULE_PATH_NAMING, RULE_TEST_PLACEMENT, RULE_UNUSED_SUPPRESSION,
};
use crate::{Severity, Violation};
use std::path::Path;

impl Violation {
    pub(crate) fn misplaced_test_file(file: &Path, severity: Severity, suggestion: String) -> Self {
        let mut violation = base_violation(
            RULE_TEST_PLACEMENT,
            severity,
            file,
            "test file is not in an allowed test location",
        );
        violation.suggestion = Some(suggestion);
        violation
    }

    pub(crate) fn path_naming(
        file: &Path,
        severity: Severity,
        message: String,
        suggestion: String,
    ) -> Self {
        let mut violation = base_violation(RULE_PATH_NAMING, severity, file, message);
        violation.suggestion = Some(suggestion);
        violation
    }

    pub(crate) fn invalid_suppression(
        file: &Path,
        severity: Severity,
        line: usize,
        column: usize,
        message: String,
    ) -> Self {
        let mut violation = base_violation(RULE_INVALID_SUPPRESSION, severity, file, message);
        violation.line = Some(line);
        violation.column = Some(column);
        violation
    }

    pub(crate) fn unused_suppression(
        file: &Path,
        severity: Severity,
        line: usize,
        column: usize,
        rule: &str,
    ) -> Self {
        let mut violation = base_violation(
            RULE_UNUSED_SUPPRESSION,
            severity,
            file,
            format!("inline suppression for {rule} does not match a violation on the next line"),
        );
        violation.line = Some(line);
        violation.column = Some(column);
        violation.suggestion =
            Some("remove the unused suppression or update the rule name".to_string());
        violation
    }
}
