use crate::RuleCatalogEntry;
use std::collections::BTreeMap;

pub fn render_rules_pretty(rules: &[RuleCatalogEntry]) -> String {
    let mut output = String::from("rules\n\n");
    for (group, entries) in grouped_rules(rules) {
        output.push_str(&group);
        output.push('\n');
        for rule in entries {
            output.push_str("  ");
            output.push_str(rule.name);
            output.push('\n');
            output.push_str("    default: ");
            output.push_str(rule.default_severity);
            output.push('\n');
            output.push_str("    architecture: ");
            output.push_str(rule.architecture_family);
            output.push('\n');
            if !rule.legacy_aliases.is_empty() {
                output.push_str("    aliases: ");
                output.push_str(&rule.legacy_aliases.join(", "));
                output.push('\n');
            }
            output.push_str("    ");
            output.push_str(rule.explanation);
            output.push('\n');
        }
        output.push('\n');
    }
    output
}

fn grouped_rules(rules: &[RuleCatalogEntry]) -> Vec<(String, Vec<&RuleCatalogEntry>)> {
    let mut groups = BTreeMap::<String, Vec<&RuleCatalogEntry>>::new();
    for rule in rules {
        groups.entry(group_name(rule)).or_default().push(rule);
    }

    let mut ordered = Vec::new();
    for architecture_group in ["cleanarch/*", "verticalslice/*"] {
        if let Some(entries) = groups.remove(architecture_group) {
            ordered.push((architecture_group.to_string(), entries));
        }
    }
    ordered.extend(groups);
    ordered
}

fn group_name(rule: &RuleCatalogEntry) -> String {
    if rule.architecture_family != "neutral" {
        return rule.architecture_family.to_string();
    }

    rule.name.split_once('/').map_or_else(
        || "neutral".to_string(),
        |(namespace, _)| format!("{namespace}/*"),
    )
}
