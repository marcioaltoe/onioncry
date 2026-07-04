use crate::{CheckReport, CheckStatus, CheckSummary, FailOn, Violation};

pub fn build_report(file_count: usize, violations: &[Violation], fail_on: FailOn) -> CheckReport {
    let warning_count = violations
        .iter()
        .filter(|violation| !violation.baselined && violation.severity == "warn")
        .count();
    let error_count = violations
        .iter()
        .filter(|violation| !violation.baselined && violation.severity == "error")
        .count();
    let baselined_count = violations
        .iter()
        .filter(|violation| violation.baselined)
        .count();
    let should_fail = match fail_on {
        FailOn::Error => error_count > 0,
        FailOn::Warning => error_count > 0 || warning_count > 0,
    };

    CheckReport {
        status: if should_fail {
            CheckStatus::Fail
        } else {
            CheckStatus::Pass
        },
        summary: CheckSummary {
            file_count,
            warning_count,
            error_count,
            violation_count: violations.len() - baselined_count,
            baselined_count,
        },
        violations: violations.to_vec(),
    }
}
