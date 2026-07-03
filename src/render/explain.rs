use super::shared::render_pretty_violation;
use crate::{BoundaryExplanation, ExplainReport};

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

fn boundary_summary(boundary: &BoundaryExplanation) -> String {
    match &boundary.name {
        Some(name) => format!("{} {}", boundary.status, name),
        None => boundary.status.clone(),
    }
}
