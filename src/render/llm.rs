use super::shared::{pretty_column, pretty_line};
use crate::{CheckReport, LLM_REPORT_METADATA, LLM_REPORT_SEPARATOR, Violation, rule_explanation};
use std::collections::BTreeMap;

pub fn render_llm(report: &CheckReport) -> String {
    let groups = llm_groups(report);
    let mut output = format!(
        "status: {}\nfilesChecked: {}\nproblemCount: {}\nerrorCount: {}\nwarningCount: {}\ngroupCount: {}\n",
        report.status.as_str(),
        report.summary.file_count,
        report.summary.violation_count,
        report.summary.error_count,
        report.summary.warning_count,
        groups.len()
    );
    if report.summary.baselined_count > 0 {
        output.push_str(&format!(
            "baselinedCount: {}\n",
            report.summary.baselined_count
        ));
    }
    if report.summary.suppressed_count > 0 {
        output.push_str(&format!(
            "suppressedCount: {}\n",
            report.summary.suppressed_count
        ));
    }

    for (index, group) in groups.iter().enumerate() {
        output.push_str(&format!(
            "\ngroup {}\nstate: {}\nactionable: {}\ncount: {}\nseverity: {}\nrule: {}\nmessage: {}\nwhy: {}\n",
            index + 1,
            group.key.state(),
            group.key.is_actionable(),
            group.violations.len(),
            group.key.severity,
            group.key.rule,
            group.key.message,
            rule_explanation(&group.key.rule)
        ));
        if let Some(import_specifier) = &group.key.import_specifier {
            output.push_str(&format!("import: {import_specifier}\n"));
        }
        if let Some(package_name) = &group.key.package_name {
            output.push_str(&format!("package: {package_name}\n"));
        }
        if let (Some(from_layer), Some(to_layer)) = (&group.key.from_layer, &group.key.to_layer) {
            output.push_str(&format!("layers: {from_layer} -> {to_layer}\n"));
        }
        if let (Some(from_context), Some(to_context)) =
            (&group.key.from_context, &group.key.to_context)
        {
            output.push_str(&format!("contexts: {from_context} -> {to_context}\n"));
        }
        if let Some(target_file) = &group.key.target_file {
            output.push_str(&format!("target: {target_file}\n"));
        }
        if let Some(cycle_path) = &group.key.cycle_path {
            output.push_str(&format!("cycle: {}\n", cycle_path.join(" -> ")));
        }
        if let Some(matched_layers) = &group.key.matched_layers {
            output.push_str(&format!("matchedLayers: {}\n", matched_layers.join(", ")));
        }
        if let Some(matched_contexts) = &group.key.matched_contexts {
            output.push_str(&format!(
                "matchedContexts: {}\n",
                matched_contexts.join(", ")
            ));
        }
        if let Some(suggestion) = &group.key.suggestion {
            output.push_str(&format!("tip: {suggestion}\n"));
        }
        if group.key.baselined {
            output.push_str(
                "baseline: fix this violation, then rerun onioncry check --write-baseline to shrink the baseline\n",
            );
        }
        if group.key.suppressed {
            if let Some(reason) = &group.key.suppression_reason {
                output.push_str(&format!("suppressionReason: {reason}\n"));
            }
            output.push_str(
                "suppression: remove the disable comment once this exception is no longer needed\n",
            );
        }
        output.push_str("locations:\n");
        for violation in &group.violations {
            output.push_str(&format!(
                "- {}:{}:{}\n",
                violation.file,
                pretty_line(violation),
                pretty_column(violation)
            ));
        }
    }

    output.push('\n');
    output.push_str(LLM_REPORT_SEPARATOR);
    output.push('\n');
    output.push_str(LLM_REPORT_METADATA);
    output.push('\n');

    output
}

struct LlmGroup<'a> {
    key: LlmGroupKey,
    violations: Vec<&'a Violation>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct LlmGroupKey {
    baselined: bool,
    suppressed: bool,
    rule: String,
    severity: String,
    message: String,
    import_specifier: Option<String>,
    package_name: Option<String>,
    from_layer: Option<String>,
    to_layer: Option<String>,
    from_context: Option<String>,
    to_context: Option<String>,
    target_file: Option<String>,
    cycle_path: Option<Vec<String>>,
    suggestion: Option<String>,
    matched_layers: Option<Vec<String>>,
    matched_contexts: Option<Vec<String>>,
    suppression_reason: Option<String>,
}

impl LlmGroupKey {
    fn from_violation(violation: &Violation) -> Self {
        Self {
            baselined: violation.baselined,
            suppressed: violation.suppressed,
            rule: violation.rule.clone(),
            severity: violation.severity.clone(),
            message: violation.message.clone(),
            import_specifier: violation.import_specifier.clone(),
            package_name: violation.package_name.clone(),
            from_layer: violation.from_layer.clone(),
            to_layer: violation.to_layer.clone(),
            from_context: violation.from_context.clone(),
            to_context: violation.to_context.clone(),
            target_file: violation.target_file.clone(),
            cycle_path: violation.cycle_path.clone(),
            suggestion: violation.suggestion.clone(),
            matched_layers: violation.matched_layers.clone(),
            matched_contexts: violation.matched_contexts.clone(),
            suppression_reason: violation.suppression_reason.clone(),
        }
    }

    fn state(&self) -> &'static str {
        if self.baselined {
            "baselined"
        } else if self.suppressed {
            "suppressed"
        } else {
            "active"
        }
    }

    fn is_actionable(&self) -> bool {
        !self.baselined && !self.suppressed
    }

    fn state_rank(&self) -> usize {
        if self.is_actionable() {
            0
        } else if self.suppressed {
            1
        } else {
            2
        }
    }
}

fn llm_groups(report: &CheckReport) -> Vec<LlmGroup<'_>> {
    let mut grouped: BTreeMap<LlmGroupKey, Vec<&Violation>> = BTreeMap::new();
    for violation in &report.violations {
        grouped
            .entry(LlmGroupKey::from_violation(violation))
            .or_default()
            .push(violation);
    }

    let mut groups = grouped
        .into_iter()
        .map(|(key, mut violations)| {
            violations.sort_by(|left, right| {
                left.file
                    .cmp(&right.file)
                    .then_with(|| pretty_line(left).cmp(&pretty_line(right)))
                    .then_with(|| pretty_column(left).cmp(&pretty_column(right)))
            });
            LlmGroup { key, violations }
        })
        .collect::<Vec<_>>();

    groups.sort_by(|left, right| {
        left.key
            .state_rank()
            .cmp(&right.key.state_rank())
            .then_with(|| {
                severity_rank(&left.key.severity).cmp(&severity_rank(&right.key.severity))
            })
            .then_with(|| right.violations.len().cmp(&left.violations.len()))
            .then_with(|| left.key.rule.cmp(&right.key.rule))
            .then_with(|| left.key.message.cmp(&right.key.message))
    });
    groups
}

fn severity_rank(severity: &str) -> usize {
    match severity {
        "error" => 0,
        "warn" => 1,
        _ => 2,
    }
}
