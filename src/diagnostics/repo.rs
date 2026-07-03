use super::builder::base_violation;
use crate::*;
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
}
