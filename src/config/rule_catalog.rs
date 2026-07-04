use super::types::{ArchitectureMode, RuleSetting, Severity};
use crate::rules::catalog::{
    ArchitectureRuleFamily, RULE_NO_FORBIDDEN_IMPORTS, canonical_rule_name, default_rule_severity,
    known_rule_names_display, rule_descriptor_for,
};
use crate::{ExternalPackageLayerPolicy, OnionCryError, PackageAllowlist, Result};
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
        let canonical_rule =
            canonical_rule_name(rule).ok_or_else(|| OnionCryError::UnknownRule {
                rule: rule.to_string(),
                expected: known_rule_names_display(),
            })?;
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

pub(crate) fn validate_architecture_rule_mode(
    mode: ArchitectureMode,
    rules: &BTreeMap<String, RuleSetting>,
) -> Result<()> {
    let expected_family = ArchitectureRuleFamily::expected_for_mode(mode);
    for (rule, setting) in rules {
        if setting.severity == Severity::Off {
            continue;
        }
        let Some(family) =
            rule_descriptor_for(rule).and_then(|descriptor| descriptor.architecture_family)
        else {
            continue;
        };
        if family == expected_family {
            continue;
        }
        return Err(OnionCryError::ArchitectureRuleModeMismatch {
            rule: rule.clone(),
            mode: mode.as_str(),
            expected_family: expected_family.display(),
        });
    }

    Ok(())
}

pub(crate) fn default_rule_setting(rule: &str) -> RuleSetting {
    RuleSetting {
        severity: default_rule_severity(rule),
        options: None,
    }
}
