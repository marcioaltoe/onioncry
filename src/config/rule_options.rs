use super::types::RuleSetting;
use crate::{OnionCryError, PackageAllowlist, Result};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashSet};

pub(crate) fn options_object<'a>(
    rule: &str,
    setting: &'a RuleSetting,
) -> Result<Option<&'a Map<String, Value>>> {
    match &setting.options {
        None => Ok(None),
        Some(Value::Object(options)) => Ok(Some(options)),
        Some(_) => Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: "expected rule options object".to_string(),
        }),
    }
}

pub(crate) fn string_vec_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[&str],
) -> Result<Vec<String>> {
    let Some(options) = options_object(rule, setting)? else {
        return Ok(default.iter().map(|value| (*value).to_string()).collect());
    };
    let Some(value) = options.get(key) else {
        return Ok(default.iter().map(|value| (*value).to_string()).collect());
    };

    match value {
        Value::String(value) => Ok(vec![value.clone()]),
        Value::Array(items) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| OnionCryError::InvalidRuleValue {
                        rule: rule.to_string(),
                        message: format!("{key} entries must be strings"),
                    })
            })
            .collect(),
        _ => Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be a string or array of strings"),
        }),
    }
}

pub(crate) fn string_set_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[&str],
) -> Result<HashSet<String>> {
    Ok(string_vec_option(rule, setting, key, default)?
        .into_iter()
        .collect())
}

pub(crate) fn package_pattern_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[&str],
) -> Result<PackageAllowlist> {
    PackageAllowlist::from_patterns(&string_vec_option(rule, setting, key, default)?)
}

pub(crate) fn suffix_map_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[(&str, &[&str])],
) -> Result<BTreeMap<String, Vec<String>>> {
    let Some(options) = options_object(rule, setting)? else {
        return Ok(default_suffix_map(default));
    };
    let Some(value) = options.get(key) else {
        return Ok(default_suffix_map(default));
    };
    let Value::Object(entries) = value else {
        return Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be an object keyed by collection directory"),
        });
    };

    entries
        .iter()
        .map(|(collection, suffixes)| {
            Ok((
                collection.to_string(),
                string_or_array_value(rule, key, suffixes)?,
            ))
        })
        .collect()
}

pub(crate) fn string_set_map_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &[(&str, &[&str])],
) -> Result<BTreeMap<String, HashSet<String>>> {
    let mut entries = default
        .iter()
        .map(|(entry_key, values)| {
            (
                (*entry_key).to_string(),
                values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<HashSet<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let Some(options) = options_object(rule, setting)? else {
        return Ok(entries);
    };
    let Some(value) = options.get(key) else {
        return Ok(entries);
    };
    let Value::Object(overrides) = value else {
        return Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be an object keyed by source area"),
        });
    };

    for (entry_key, entry_value) in overrides {
        entries.insert(
            entry_key.to_string(),
            string_or_array_value(rule, key, entry_value)?
                .into_iter()
                .collect(),
        );
    }

    Ok(entries)
}

fn default_suffix_map(default: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    default
        .iter()
        .map(|(collection, suffixes)| {
            (
                (*collection).to_string(),
                suffixes
                    .iter()
                    .map(|suffix| (*suffix).to_string())
                    .collect(),
            )
        })
        .collect()
}

fn string_or_array_value(rule: &str, key: &str, value: &Value) -> Result<Vec<String>> {
    match value {
        Value::String(value) => Ok(vec![value.clone()]),
        Value::Array(items) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| OnionCryError::InvalidRuleValue {
                        rule: rule.to_string(),
                        message: format!("{key} entries must be strings"),
                    })
            })
            .collect(),
        _ => Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} values must be strings or arrays of strings"),
        }),
    }
}

pub(crate) fn usize_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: usize,
) -> Result<usize> {
    let Some(options) = options_object(rule, setting)? else {
        return Ok(default);
    };
    let Some(value) = options.get(key) else {
        return Ok(default);
    };
    let Some(value) = value.as_u64() else {
        return Err(OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be a positive integer"),
        });
    };
    usize::try_from(value).map_err(|_| OnionCryError::InvalidRuleValue {
        rule: rule.to_string(),
        message: format!("{key} is too large"),
    })
}

pub(crate) fn bool_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: bool,
) -> Result<bool> {
    let Some(options) = options_object(rule, setting)? else {
        return Ok(default);
    };
    let Some(value) = options.get(key) else {
        return Ok(default);
    };
    value
        .as_bool()
        .ok_or_else(|| OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be true or false"),
        })
}

pub(crate) fn string_option(
    rule: &str,
    setting: &RuleSetting,
    key: &str,
    default: &str,
) -> Result<String> {
    let Some(options) = options_object(rule, setting)? else {
        return Ok(default.to_string());
    };
    let Some(value) = options.get(key) else {
        return Ok(default.to_string());
    };
    value
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| OnionCryError::InvalidRuleValue {
            rule: rule.to_string(),
            message: format!("{key} must be a string"),
        })
}
