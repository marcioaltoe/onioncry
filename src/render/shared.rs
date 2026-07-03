use crate::{Violation, rule_explanation};

pub(super) fn render_pretty_violation(violation: &Violation, include_tips: bool) -> String {
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

    output.push_str(&format!("    why: {}\n", rule_explanation(&violation.rule)));
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

pub(super) fn pluralize<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

pub(super) fn pretty_line(violation: &Violation) -> usize {
    violation.line.unwrap_or(1)
}

pub(super) fn pretty_column(violation: &Violation) -> usize {
    violation.column.unwrap_or(1)
}

fn pretty_severity(severity: &str) -> &str {
    match severity {
        "warn" => "warning",
        other => other,
    }
}
