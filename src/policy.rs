use crate::*;
use globset::GlobSet;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) struct RulePolicy {
    base_rules: BTreeMap<String, RuleSetting>,
    overrides: Vec<CompiledOverride>,
}

struct CompiledOverride {
    files: GlobSet,
    rules: BTreeMap<String, RuleSetting>,
}

pub(crate) struct ExternalPackagePolicy {
    default_severity: Severity,
    default_allow: PackageAllowlist,
    layers: Vec<ExternalPackageLayerPolicy>,
}

pub(crate) struct ExternalPackageLayerPolicy {
    pub(crate) from_layer: String,
    pub(crate) severity: Severity,
    pub(crate) allow: PackageAllowlist,
}

pub(crate) struct EffectiveExternalPackageLayerPolicy<'a> {
    pub(crate) severity: Severity,
    pub(crate) allow: &'a PackageAllowlist,
}

pub(crate) struct PackageAllowlist {
    patterns: GlobSet,
}

impl RulePolicy {
    pub(crate) fn new(config: &Config) -> Result<Self> {
        let base_rules = parse_rule_map(&config.rules)?;
        validate_architecture_rule_mode(config.architecture.mode, &base_rules)?;
        let mut overrides = Vec::new();

        for override_config in &config.overrides {
            let rules = parse_rule_map(&override_config.rules)?;
            validate_architecture_rule_mode(config.architecture.mode, &rules)?;
            overrides.push(CompiledOverride {
                files: build_glob_set(&override_config.files)?,
                rules,
            });
        }

        Ok(Self {
            base_rules,
            overrides,
        })
    }

    pub(crate) fn effective_severity(
        &self,
        rule: &str,
        project_root: &Path,
        file: &Path,
    ) -> Severity {
        self.effective_rule(rule, project_root, file).severity
    }

    pub(crate) fn effective_rule(
        &self,
        rule: &str,
        project_root: &Path,
        file: &Path,
    ) -> RuleSetting {
        let relative_path = file.strip_prefix(project_root).unwrap_or(file);
        let mut setting = self
            .base_rules
            .get(rule)
            .cloned()
            .unwrap_or_else(|| default_rule_setting(rule));

        for override_config in &self.overrides {
            if !override_config.files.is_match(relative_path) {
                continue;
            }
            if let Some(override_setting) = override_config.rules.get(rule) {
                setting = override_setting.clone();
            }
        }

        setting
    }
}

impl ExternalPackagePolicy {
    pub(crate) fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let mut policy = Self {
            default_severity: setting.severity,
            default_allow: PackageAllowlist::empty()?,
            layers: Vec::new(),
        };

        let Some(options) = &setting.options else {
            return Ok(policy);
        };

        match options {
            Value::Object(options) => {
                if let Some(allow) = options.get("allow") {
                    policy.default_allow = PackageAllowlist::from_value(allow)?;
                }
                if let Some(layers) = options.get("layers") {
                    policy.layers =
                        parse_external_package_layer_policies(layers, setting.severity)?;
                }
            }
            Value::Array(_) => {
                policy.layers = parse_external_package_layer_policies(options, setting.severity)?;
            }
            _ => {
                return Err(OnionCryError::InvalidRuleValue {
                    rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
                    message: "expected options object or layer policy array".to_string(),
                });
            }
        }

        Ok(policy)
    }

    pub(crate) fn for_layer(&self, from_layer: &str) -> EffectiveExternalPackageLayerPolicy<'_> {
        let mut effective = EffectiveExternalPackageLayerPolicy {
            severity: self.default_severity,
            allow: &self.default_allow,
        };

        for layer in &self.layers {
            if layer.from_layer == from_layer {
                effective = EffectiveExternalPackageLayerPolicy {
                    severity: layer.severity,
                    allow: &layer.allow,
                };
            }
        }

        effective
    }
}

impl PackageAllowlist {
    pub(crate) fn empty() -> Result<Self> {
        Self::from_patterns(&[])
    }

    pub(crate) fn from_value(value: &Value) -> Result<Self> {
        let patterns = match value {
            Value::String(pattern) => vec![pattern.clone()],
            Value::Array(items) => items
                .iter()
                .map(|item| {
                    item.as_str().map(str::to_string).ok_or_else(|| {
                        OnionCryError::InvalidRuleValue {
                            rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
                            message: "allow entries must be package pattern strings".to_string(),
                        }
                    })
                })
                .collect::<Result<Vec<_>>>()?,
            _ => {
                return Err(OnionCryError::InvalidRuleValue {
                    rule: RULE_NO_FORBIDDEN_IMPORTS.to_string(),
                    message: "allow must be a package pattern string or array of strings"
                        .to_string(),
                });
            }
        };

        Self::from_patterns(&patterns)
    }

    pub(crate) fn from_patterns(patterns: &[String]) -> Result<Self> {
        Ok(Self {
            patterns: build_glob_set(patterns)?,
        })
    }

    pub(crate) fn is_allowed(&self, package_name: &str) -> bool {
        self.patterns.is_match(package_name)
    }
}
