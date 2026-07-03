use super::types::{ArchitectureMode, RuleSetting, Severity};
use crate::*;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

impl Severity {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Severity::Off => "off",
            Severity::Warn => "warn",
            Severity::Error => "error",
        }
    }
}

impl ArchitectureMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::CleanArchitecture => "cleanArchitecture",
            Self::VerticalSlice => "verticalSlice",
        }
    }

    fn expected_rule_family(self) -> &'static str {
        match self {
            Self::CleanArchitecture => "cleanarch/*",
            Self::VerticalSlice => "verticalslice/*",
        }
    }
}

fn severity_from_str(value: &str) -> Option<Severity> {
    match value {
        "off" => Some(Severity::Off),
        "warn" => Some(Severity::Warn),
        "error" => Some(Severity::Error),
        _ => None,
    }
}

pub(crate) fn parse_rule_map(rules: &Map<String, Value>) -> Result<BTreeMap<String, RuleSetting>> {
    let mut parsed_rules = BTreeMap::new();
    for (rule, value) in rules {
        let canonical_rule = canonical_rule_name(rule)?;
        parsed_rules.insert(canonical_rule.to_string(), parse_rule_setting(rule, value)?);
    }
    Ok(parsed_rules)
}

fn parse_rule_setting(rule: &str, value: &Value) -> Result<RuleSetting> {
    let (severity_text, options) = match value {
        Value::String(severity) => (severity.as_str(), None),
        Value::Array(items) => {
            let severity = items.first().and_then(Value::as_str).ok_or_else(|| {
                OnionCryError::InvalidRuleValue {
                    rule: rule.to_string(),
                    message: "expected [severity, options] with severity off, warn, or error"
                        .to_string(),
                }
            })?;
            if items.len() > 2 {
                return Err(OnionCryError::InvalidRuleValue {
                    rule: rule.to_string(),
                    message: "expected [severity, options] with at most two entries".to_string(),
                });
            }
            (severity, items.get(1).cloned())
        }
        _ => {
            return Err(OnionCryError::InvalidRuleValue {
                rule: rule.to_string(),
                message: "expected severity string or [severity, options]".to_string(),
            });
        }
    };

    let severity =
        severity_from_str(severity_text).ok_or_else(|| OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("invalid severity {severity_text:?}; expected off, warn, or error"),
        })?;

    Ok(RuleSetting { severity, options })
}

pub(crate) fn parse_external_package_layer_policies(
    value: &Value,
    default_severity: Severity,
) -> Result<Vec<ExternalPackageLayerPolicy>> {
    let Value::Array(items) = value else {
        return Err(OnionCryError::InvalidRuleValue {
            rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
            message: "layers must be an array of layer policy objects".to_string(),
        });
    };

    items
        .iter()
        .map(|item| parse_external_package_layer_policy(item, default_severity))
        .collect()
}

fn parse_external_package_layer_policy(
    value: &Value,
    default_severity: Severity,
) -> Result<ExternalPackageLayerPolicy> {
    let Value::Object(entry) = value else {
        return Err(OnionCryError::InvalidRuleValue {
            rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
            message: "layer policy entries must be objects".to_string(),
        });
    };
    let from_layer = entry
        .get("fromLayer")
        .and_then(Value::as_str)
        .ok_or_else(|| OnionCryError::InvalidRuleValue {
            rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
            message: "layer policy entries require fromLayer".to_string(),
        })?;
    let severity = entry
        .get("severity")
        .and_then(Value::as_str)
        .map(|severity| {
            severity_from_str(severity).ok_or_else(|| OnionCryError::InvalidRuleValue {
                rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
                message: format!(
                    "invalid layer severity {severity:?}; expected off, warn, or error"
                ),
            })
        })
        .transpose()?
        .unwrap_or(default_severity);
    let allow = entry
        .get("allow")
        .map(PackageAllowlist::from_value)
        .transpose()?
        .unwrap_or(PackageAllowlist::empty()?);

    Ok(ExternalPackageLayerPolicy {
        from_layer: from_layer.to_string(),
        severity,
        allow,
    })
}

fn canonical_rule_name(rule: &str) -> Result<&'static str> {
    let canonical = match rule {
        RULE_UNCLASSIFIED_FILE | "onion/unclassified-file" => RULE_UNCLASSIFIED_FILE,
        RULE_AMBIGUOUS_LAYER | "onion/ambiguous-layer" => RULE_AMBIGUOUS_LAYER,
        RULE_AMBIGUOUS_CONTEXT | "onion/ambiguous-context" => RULE_AMBIGUOUS_CONTEXT,
        RULE_NO_LAYER_LEAK | "onion/no-layer-leak" => RULE_NO_LAYER_LEAK,
        RULE_NO_FORBIDDEN_IMPORTS | "onion/no-forbidden-imports" => RULE_NO_FORBIDDEN_IMPORTS,
        RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT | "onion/no-cross-context-internal-import" => {
            RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT
        }
        RULE_NO_FRAMEWORK_IN_CORE | "onion/no-framework-in-core" => RULE_NO_FRAMEWORK_IN_CORE,
        RULE_NO_OUTER_DATA_FORMAT_IN_CORE | "onion/no-outer-data-format-in-core" => {
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE
        }
        RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT | "onion/no-public-surface-internal-reexport" => {
            RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT
        }
        RULE_NO_CONTEXT_CYCLE | "onion/no-context-cycle" => RULE_NO_CONTEXT_CYCLE,
        RULE_NO_UNOWNED_SCHEMA_IMPORT | "onion/no-unowned-schema-import" => {
            RULE_NO_UNOWNED_SCHEMA_IMPORT
        }
        RULE_CLEAN_ARTIFACT_PLACEMENT => RULE_CLEAN_ARTIFACT_PLACEMENT,
        RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT => {
            RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT
        }
        RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS => RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS,
        RULE_VERTICAL_SLICE_ENTRY_POINT => RULE_VERTICAL_SLICE_ENTRY_POINT,
        RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS => RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS,
        RULE_NO_CONCRETE_DEPENDENCY | "onion/no-concrete-dependency" => RULE_NO_CONCRETE_DEPENDENCY,
        RULE_FEATURE_ENVY => RULE_FEATURE_ENVY,
        RULE_SHOTGUN_SURGERY => RULE_SHOTGUN_SURGERY,
        RULE_TEST_PLACEMENT => RULE_TEST_PLACEMENT,
        RULE_PATH_NAMING => RULE_PATH_NAMING,
        RULE_FEATURE_SYSTEM_LAYOUT => RULE_FEATURE_SYSTEM_LAYOUT,
        RULE_FEATURE_SYSTEM_PUBLIC_API => RULE_FEATURE_SYSTEM_PUBLIC_API,
        RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW => RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
        RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT => RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
        RULE_FEATURE_SYSTEM_QUERY_CONTRACT => RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
        _ => {
            return Err(OnionCryError::UnknownRule {
                rule: rule.to_string(),
                expected: KNOWN_RULE_NAMES_DISPLAY,
            });
        }
    };

    Ok(canonical)
}

pub(crate) fn validate_architecture_rule_mode(
    mode: ArchitectureMode,
    rules: &BTreeMap<String, RuleSetting>,
) -> Result<()> {
    for (rule, setting) in rules {
        if setting.severity == Severity::Off {
            continue;
        }
        let Some(family) = architecture_rule_family(rule) else {
            continue;
        };
        if family == mode.expected_rule_family() {
            continue;
        }
        return Err(OnionCryError::ArchitectureRuleModeMismatch {
            rule: rule.clone(),
            mode: mode.as_str(),
            expected_family: mode.expected_rule_family(),
        });
    }

    Ok(())
}

fn architecture_rule_family(rule: &str) -> Option<&'static str> {
    if rule.starts_with("cleanarch/") {
        Some("cleanarch/*")
    } else if rule.starts_with("verticalslice/") {
        Some("verticalslice/*")
    } else {
        None
    }
}

pub(crate) fn default_rule_setting(rule: &str) -> RuleSetting {
    RuleSetting {
        severity: default_rule_severity(rule),
        options: None,
    }
}

fn default_rule_severity(rule: &str) -> Severity {
    match rule {
        RULE_NO_LAYER_LEAK => Severity::Error,
        RULE_AMBIGUOUS_LAYER => Severity::Error,
        RULE_AMBIGUOUS_CONTEXT => Severity::Error,
        RULE_NO_FORBIDDEN_IMPORTS => Severity::Error,
        RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT => Severity::Error,
        RULE_UNCLASSIFIED_FILE => Severity::Warn,
        RULE_NO_FRAMEWORK_IN_CORE
        | RULE_NO_OUTER_DATA_FORMAT_IN_CORE
        | RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT
        | RULE_NO_CONTEXT_CYCLE
        | RULE_NO_UNOWNED_SCHEMA_IMPORT
        | RULE_CLEAN_ARTIFACT_PLACEMENT
        | RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT
        | RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS
        | RULE_VERTICAL_SLICE_ENTRY_POINT
        | RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS
        | RULE_NO_CONCRETE_DEPENDENCY
        | RULE_FEATURE_ENVY
        | RULE_SHOTGUN_SURGERY
        | RULE_TEST_PLACEMENT
        | RULE_PATH_NAMING
        | RULE_FEATURE_SYSTEM_LAYOUT
        | RULE_FEATURE_SYSTEM_PUBLIC_API
        | RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW
        | RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT
        | RULE_FEATURE_SYSTEM_QUERY_CONTRACT => Severity::Off,
        _ => Severity::Warn,
    }
}
