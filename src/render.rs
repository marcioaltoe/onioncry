use crate::*;
use std::collections::BTreeMap;

pub fn build_report(file_count: usize, violations: &[Violation], fail_on: FailOn) -> CheckReport {
    let warning_count = violations
        .iter()
        .filter(|violation| violation.severity == "warn")
        .count();
    let error_count = violations
        .iter()
        .filter(|violation| violation.severity == "error")
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
            violation_count: violations.len(),
        },
        violations: violations.to_vec(),
    }
}

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

pub fn render_explain_pretty(report: &ExplainReport, include_tips: bool) -> String {
    let mut output = format!(
        "file: {}\nlayer: {}\ncontext: {}\npublicSurface: {}\n",
        report.file,
        boundary_summary(&report.layer),
        boundary_summary(&report.context),
        report.public_surface
    );

    output.push_str("imports:\n");
    for import in &report.imports {
        output.push_str(&format!(
            "- {} {} {}:{}",
            import.resolution, import.specifier, import.line, import.column
        ));
        if let Some(target_file) = &import.target_file {
            output.push_str(&format!(" -> {target_file}"));
        }
        if let Some(package_name) = &import.package_name {
            output.push_str(&format!(" package {package_name}"));
        }
        if let Some(package_allowed) = import.package_allowed {
            output.push_str(&format!(" allowed {package_allowed}"));
        }
        output.push('\n');
    }

    output.push_str("violations:\n");
    for violation in &report.violations {
        output.push_str(&render_pretty_violation(violation, include_tips));
    }

    output
}

fn render_pretty_summary(report: &CheckReport) -> String {
    format!(
        "{} {} ({} {}, {} {})\n{} {}\nstatus: {}\n",
        report.summary.violation_count,
        pluralize(report.summary.violation_count, "problem", "problems"),
        report.summary.error_count,
        pluralize(report.summary.error_count, "error", "errors"),
        report.summary.warning_count,
        pluralize(report.summary.warning_count, "warning", "warnings"),
        report.summary.file_count,
        pluralize(report.summary.file_count, "file checked", "files checked"),
        report.status.as_str()
    )
}

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

    for (index, group) in groups.iter().enumerate() {
        output.push_str(&format!(
            "\ngroup {}\ncount: {}\nseverity: {}\nrule: {}\nmessage: {}\nwhy: {}\n",
            index + 1,
            group.violations.len(),
            group.key.severity,
            group.key.rule,
            group.key.message,
            violation_rule_explanation(&group.key.rule)
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
}

impl LlmGroupKey {
    fn from_violation(violation: &Violation) -> Self {
        Self {
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
        severity_rank(&left.key.severity)
            .cmp(&severity_rank(&right.key.severity))
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

fn render_pretty_violation(violation: &Violation, include_tips: bool) -> String {
    let mut output = format!(
        "  {}:{}  {:<7} {}  {}\n",
        pretty_line(violation),
        pretty_column(violation),
        pretty_severity(&violation.severity),
        violation.message,
        violation.rule
    );

    if !include_tips {
        return output;
    }

    output.push_str(&format!(
        "    why: {}\n",
        violation_rule_explanation(&violation.rule)
    ));
    if let Some(import_specifier) = &violation.import_specifier {
        output.push_str(&format!("    import: {import_specifier}\n"));
    }
    if let Some(package_name) = &violation.package_name {
        output.push_str(&format!("    package: {package_name}\n"));
    }
    if let (Some(from_layer), Some(to_layer)) = (&violation.from_layer, &violation.to_layer) {
        output.push_str(&format!("    layers: {from_layer} -> {to_layer}\n"));
    }
    if let (Some(from_context), Some(to_context)) = (&violation.from_context, &violation.to_context)
    {
        output.push_str(&format!("    contexts: {from_context} -> {to_context}\n"));
    }
    if let Some(target_file) = &violation.target_file {
        output.push_str(&format!("    target: {target_file}\n"));
    }
    if let Some(cycle_path) = &violation.cycle_path {
        output.push_str(&format!("    cycle: {}\n", cycle_path.join(" -> ")));
    }
    if let Some(matched_layers) = &violation.matched_layers {
        output.push_str(&format!(
            "    matched layers: {}\n",
            matched_layers.join(", ")
        ));
    }
    if let Some(matched_contexts) = &violation.matched_contexts {
        output.push_str(&format!(
            "    matched contexts: {}\n",
            matched_contexts.join(", ")
        ));
    }
    if let Some(suggestion) = &violation.suggestion {
        output.push_str(&format!("    tip: {suggestion}\n"));
    }

    output
}

fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

fn pretty_line(violation: &Violation) -> usize {
    violation.line.unwrap_or(1)
}

fn pretty_column(violation: &Violation) -> usize {
    violation.column.unwrap_or(1)
}

fn pretty_severity(severity: &str) -> &str {
    match severity {
        "warn" => "warning",
        other => other,
    }
}

fn violation_rule_explanation(rule: &str) -> &'static str {
    match rule {
        RULE_UNCLASSIFIED_FILE => {
            "Layer checks need each analyzed file to match exactly one configured architectural layer."
        }
        RULE_AMBIGUOUS_LAYER => {
            "Overlapping layer patterns make it unclear which dependency policy applies to this file."
        }
        RULE_AMBIGUOUS_CONTEXT => {
            "Overlapping context patterns make it unclear which ownership boundary applies to this file."
        }
        RULE_NO_LAYER_LEAK => {
            "Layer rules only allow imports declared in the importing layer's mayImport policy."
        }
        RULE_NO_FORBIDDEN_IMPORTS => {
            "External packages are closed by default in sensitive layers unless explicitly allowed."
        }
        RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT => {
            "Cross-context imports must target the imported context's configured public surface."
        }
        RULE_NO_FRAMEWORK_IN_CORE => "Core layers should depend on ports, not framework packages.",
        RULE_NO_OUTER_DATA_FORMAT_IN_CORE => {
            "Core layers should not mention data formats owned by outer details."
        }
        RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT => {
            "A public surface should expose intentional contracts, not internal implementation files."
        }
        RULE_NO_CONTEXT_CYCLE => {
            "Context dependencies should form a directed acyclic ownership graph."
        }
        RULE_NO_UNOWNED_SCHEMA_IMPORT => {
            "A context should not depend directly on another context's storage schema."
        }
        RULE_CLEAN_ARTIFACT_PLACEMENT => {
            "Clean Architecture artifacts should live under a context-first layer boundary or a contextless base layer."
        }
        RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT => {
            "Cross-slice imports should target the imported slice's configured public surface."
        }
        RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS => {
            "Vertical Slice artifacts should live under the configured slice root unless their global folder is explicitly allowed."
        }
        RULE_VERTICAL_SLICE_ENTRY_POINT => {
            "Each Vertical Slice should expose a small configured entry point so routes, jobs, or composition code depend on the slice boundary."
        }
        RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS => {
            "Vertical Slice projects should not rebuild shared technical layers such as global services, repositories, handlers, or use cases."
        }
        RULE_NO_CONCRETE_DEPENDENCY => {
            "Core layers should depend on abstractions rather than concrete details."
        }
        RULE_FEATURE_ENVY => {
            "A file that mostly imports another context may contain behavior owned by that context."
        }
        RULE_SHOTGUN_SURGERY => {
            "Files that repeatedly change with many companions may hide scattered responsibilities."
        }
        RULE_TEST_PLACEMENT => {
            "Source-level unit tests should live in colocated test directories, while integration and e2e tests should live under their dedicated workspace roots."
        }
        RULE_PATH_NAMING => {
            "Path naming checks observable file and directory names, not code symbols."
        }
        RULE_FEATURE_SYSTEM_LAYOUT => {
            "Feature system layout checks observable systems/<domain> folders, shared UI roots, and surface CSS placement."
        }
        RULE_FEATURE_SYSTEM_PUBLIC_API => {
            "Feature system public APIs should be explicit barrels, and callers outside a system should depend on those barrels instead of internals."
        }
        RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW => {
            "Feature system dependency flow keeps upper UI layers from shortcutting into adapters and keeps routes on public barrels."
        }
        RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT => {
            "Feature system adapter contracts check domain-named API adapters, typed API errors, cancellable reads, and adapter import boundaries."
        }
        RULE_FEATURE_SYSTEM_QUERY_CONTRACT => {
            "Feature system query contracts keep TanStack Query keys, options, hooks, and cache mutations owned by the system lib layer."
        }
        _ => "This finding violates the configured OnionCry architecture policy.",
    }
}

fn boundary_summary(boundary: &BoundaryExplanation) -> String {
    match &boundary.name {
        Some(name) => format!("{} {}", boundary.status, name),
        None => boundary.status.clone(),
    }
}
