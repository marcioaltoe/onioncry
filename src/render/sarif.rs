use crate::{CheckReport, RuleCatalogEntry, Violation};
use serde::Serialize;

const SARIF_SCHEMA_URI: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/cs01/schemas/sarif-schema-2.1.0.json";
const SARIF_VERSION: &str = "2.1.0";

pub fn render_sarif(
    report: &CheckReport,
    rules: &[RuleCatalogEntry],
) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&SarifLog::from_report(report, rules))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLog<'a> {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun<'a>>,
}

impl<'a> SarifLog<'a> {
    fn from_report(report: &'a CheckReport, rules: &'a [RuleCatalogEntry]) -> Self {
        Self {
            schema: SARIF_SCHEMA_URI,
            version: SARIF_VERSION,
            runs: vec![SarifRun::from_report(report, rules)],
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRun<'a> {
    tool: SarifTool<'a>,
    results: Vec<SarifResult<'a>>,
}

impl<'a> SarifRun<'a> {
    fn from_report(report: &'a CheckReport, rules: &'a [RuleCatalogEntry]) -> Self {
        Self {
            tool: SarifTool {
                driver: SarifToolComponent::from_rules(rules),
            },
            results: report
                .violations
                .iter()
                .map(|violation| SarifResult::from_violation(violation, rules))
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifTool<'a> {
    driver: SarifToolComponent<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifToolComponent<'a> {
    name: &'static str,
    semantic_version: &'static str,
    rules: Vec<SarifReportingDescriptor<'a>>,
}

impl<'a> SarifToolComponent<'a> {
    fn from_rules(rules: &'a [RuleCatalogEntry]) -> Self {
        Self {
            name: "OnionCry",
            semantic_version: env!("CARGO_PKG_VERSION"),
            rules: rules
                .iter()
                .map(SarifReportingDescriptor::from_rule)
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifReportingDescriptor<'a> {
    id: &'a str,
    name: &'a str,
    short_description: SarifMessage<'a>,
    full_description: SarifMessage<'a>,
    default_configuration: SarifReportingConfiguration,
}

impl<'a> SarifReportingDescriptor<'a> {
    fn from_rule(rule: &'a RuleCatalogEntry) -> Self {
        Self {
            id: rule.name,
            name: rule.name,
            short_description: SarifMessage { text: rule.name },
            full_description: SarifMessage {
                text: rule.explanation,
            },
            default_configuration: SarifReportingConfiguration {
                level: sarif_level(rule.default_severity),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifReportingConfiguration {
    level: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult<'a> {
    rule_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule_index: Option<usize>,
    level: &'static str,
    message: SarifMessage<'a>,
    locations: Vec<SarifLocation<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    suppressions: Vec<SarifSuppression<'a>>,
}

impl<'a> SarifResult<'a> {
    fn from_violation(violation: &'a Violation, rules: &'a [RuleCatalogEntry]) -> Self {
        Self {
            rule_id: &violation.rule,
            rule_index: rules.iter().position(|rule| rule.name == violation.rule),
            level: sarif_level(&violation.severity),
            message: SarifMessage {
                text: &violation.message,
            },
            locations: vec![SarifLocation::from_violation(violation)],
            suppressions: sarif_suppressions(violation),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLocation<'a> {
    physical_location: SarifPhysicalLocation<'a>,
}

impl<'a> SarifLocation<'a> {
    fn from_violation(violation: &'a Violation) -> Self {
        Self {
            physical_location: SarifPhysicalLocation {
                artifact_location: SarifArtifactLocation {
                    uri: &violation.file,
                },
                region: sarif_region(violation),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifPhysicalLocation<'a> {
    artifact_location: SarifArtifactLocation<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<SarifRegion>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifArtifactLocation<'a> {
    uri: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRegion {
    start_line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_column: Option<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifSuppression<'a> {
    kind: &'static str,
    state: &'static str,
    justification: &'a str,
}

impl<'a> SarifSuppression<'a> {
    fn external_baseline() -> Self {
        Self {
            kind: "external",
            state: "accepted",
            justification: "Baselined by OnionCry violation baseline.",
        }
    }

    fn in_source(reason: &'a str) -> Self {
        Self {
            kind: "inSource",
            state: "accepted",
            justification: reason,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifMessage<'a> {
    text: &'a str,
}

fn sarif_region(violation: &Violation) -> Option<SarifRegion> {
    violation.line.map(|line| SarifRegion {
        start_line: line,
        start_column: violation.column,
    })
}

fn sarif_suppressions(violation: &Violation) -> Vec<SarifSuppression<'_>> {
    if violation.baselined {
        return vec![SarifSuppression::external_baseline()];
    }

    if violation.suppressed {
        return vec![SarifSuppression::in_source(
            violation
                .suppression_reason
                .as_deref()
                .unwrap_or("Suppressed by inline OnionCry comment."),
        )];
    }

    Vec::new()
}

fn sarif_level(severity: &str) -> &'static str {
    match severity {
        "error" => "error",
        "warn" => "warning",
        "off" => "none",
        _ => "note",
    }
}
