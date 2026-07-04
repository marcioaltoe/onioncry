use super::shared::{pluralize, pretty_column, pretty_line, render_pretty_violation};
use crate::{CheckReport, Violation};

pub fn render_pretty(report: &CheckReport, include_tips: bool) -> String {
    let mut output = String::new();

    let mut current_file: Option<&str> = None;
    for violation in sorted_violations(&report.violations) {
        if current_file != Some(violation.file.as_str()) {
            if current_file.is_some() {
                output.push('\n');
            }
            current_file = Some(violation.file.as_str());
            output.push_str(&violation.file);
            output.push('\n');
        }
        output.push_str(&render_pretty_violation(violation, include_tips));
    }
    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str(&render_pretty_summary(report));
    output
}

fn render_pretty_summary(report: &CheckReport) -> String {
    let baseline_fragment = if report.summary.baselined_count > 0 {
        format!(", {} baselined", report.summary.baselined_count)
    } else {
        String::new()
    };

    format!(
        "{} {} ({} {}, {} {}{})\n{} {}\nstatus: {}\n",
        report.summary.violation_count,
        pluralize(report.summary.violation_count, "problem", "problems"),
        report.summary.error_count,
        pluralize(report.summary.error_count, "error", "errors"),
        report.summary.warning_count,
        pluralize(report.summary.warning_count, "warning", "warnings"),
        baseline_fragment,
        report.summary.file_count,
        pluralize(report.summary.file_count, "file checked", "files checked"),
        report.status.as_str()
    )
}

fn sorted_violations(violations: &[Violation]) -> Vec<&Violation> {
    let mut sorted = violations.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then_with(|| pretty_line(left).cmp(&pretty_line(right)))
            .then_with(|| pretty_column(left).cmp(&pretty_column(right)))
            .then_with(|| left.rule.cmp(&right.rule))
    });
    sorted
}
