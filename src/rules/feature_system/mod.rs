mod adapter_contract;
mod dependency_flow;
mod helpers;
mod layout;
mod location;
mod public_api;
mod query_contract;

use adapter_contract::FeatureSystemAdapterContractPolicy;
use dependency_flow::FeatureSystemDependencyFlowPolicy;
use layout::FeatureSystemLayoutPolicy;
pub(crate) use location::{FeatureSystemDependencyArea, FeatureSystemLocation};
use public_api::FeatureSystemPublicApiPolicy;
use query_contract::FeatureSystemQueryContractPolicy;

use crate::rules::catalog::{
    RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT, RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
    RULE_FEATURE_SYSTEM_LAYOUT, RULE_FEATURE_SYSTEM_PUBLIC_API, RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
};
use crate::{ImportEdge, Result, RulePolicy, RuleSetting, Severity, Violation};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// Policies need the complete graph, even when a related file uses another setting.
// Only diagnostic ownership is scoped; equal settings share one policy evaluation.
fn collect_for_effective_settings(
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
    rule: &str,
    mut evaluate: impl FnMut(&RuleSetting) -> Result<Vec<Violation>>,
) -> Result<Vec<Violation>> {
    let settings_by_file = files
        .iter()
        .map(|file| {
            (
                file.display().to_string(),
                rule_policy.effective_rule(rule, project_root, file),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut distinct_settings = Vec::new();
    let mut violations = Vec::new();

    for setting in settings_by_file.values() {
        if setting.severity == Severity::Off || distinct_settings.contains(&setting) {
            continue;
        }
        distinct_settings.push(setting);
        violations.extend(
            evaluate(setting)?
                .into_iter()
                .filter(|violation| settings_by_file.get(&violation.file) == Some(setting)),
        );
    }

    Ok(violations)
}

pub(crate) fn collect_feature_system_layout_violations(
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    collect_for_effective_settings(
        project_root,
        files,
        rule_policy,
        RULE_FEATURE_SYSTEM_LAYOUT,
        |setting| {
            let policy = FeatureSystemLayoutPolicy::from_rule_setting(setting)?;
            Ok(policy.violations(project_root, files, setting.severity))
        },
    )
}

pub(crate) fn collect_feature_system_public_api_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    collect_for_effective_settings(
        project_root,
        files,
        rule_policy,
        RULE_FEATURE_SYSTEM_PUBLIC_API,
        |setting| {
            let policy = FeatureSystemPublicApiPolicy::from_rule_setting(setting)?;
            policy.violations(project_root, files, edges, setting.severity)
        },
    )
}

pub(crate) fn collect_feature_system_dependency_flow_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    collect_for_effective_settings(
        project_root,
        files,
        rule_policy,
        RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
        |setting| {
            let policy = FeatureSystemDependencyFlowPolicy::from_rule_setting(setting)?;
            Ok(policy.violations(project_root, edges, setting.severity))
        },
    )
}

pub(crate) fn collect_feature_system_adapter_contract_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    collect_for_effective_settings(
        project_root,
        files,
        rule_policy,
        RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
        |setting| {
            let policy = FeatureSystemAdapterContractPolicy::from_rule_setting(setting)?;
            policy.violations(project_root, files, edges, setting.severity)
        },
    )
}

pub(crate) fn collect_feature_system_query_contract_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    collect_for_effective_settings(
        project_root,
        files,
        rule_policy,
        RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
        |setting| {
            let policy = FeatureSystemQueryContractPolicy::from_rule_setting(setting)?;
            policy.violations(project_root, files, edges, setting.severity)
        },
    )
}
