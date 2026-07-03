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
use crate::{ImportEdge, Result, RulePolicy, Severity, Violation};
use std::path::{Path, PathBuf};

pub(crate) fn collect_feature_system_layout_violations(
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting = rule_policy.effective_rule(RULE_FEATURE_SYSTEM_LAYOUT, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemLayoutPolicy::from_rule_setting(&rule_setting)?;

    Ok(policy.violations(project_root, files, rule_setting.severity))
}

pub(crate) fn collect_feature_system_public_api_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_PUBLIC_API, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemPublicApiPolicy::from_rule_setting(&rule_setting)?;

    policy.violations(project_root, files, edges, rule_setting.severity)
}

pub(crate) fn collect_feature_system_dependency_flow_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemDependencyFlowPolicy::from_rule_setting(&rule_setting)?;

    Ok(policy.violations(project_root, edges, rule_setting.severity))
}

pub(crate) fn collect_feature_system_adapter_contract_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemAdapterContractPolicy::from_rule_setting(&rule_setting)?;

    policy.violations(project_root, files, edges, rule_setting.severity)
}

pub(crate) fn collect_feature_system_query_contract_violations(
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let Some(rule_setting) = files.iter().find_map(|file| {
        let setting =
            rule_policy.effective_rule(RULE_FEATURE_SYSTEM_QUERY_CONTRACT, project_root, file);
        (setting.severity != Severity::Off).then_some(setting)
    }) else {
        return Ok(Vec::new());
    };
    let policy = FeatureSystemQueryContractPolicy::from_rule_setting(&rule_setting)?;

    policy.violations(project_root, files, edges, rule_setting.severity)
}
