mod support;

use support::*;

// Suite: rules command
// Invariant: onioncry rules exposes every built-in catalog rule without reading project config.
// Boundary IN: public onioncry CLI output and exit codes.
// Boundary OUT: individual rule evaluators, covered by rule-specific integration tests.

#[test]
fn rules_pretty_output_groups_catalog_rules_without_config() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    let output = onioncry()
        .current_dir(workspace.path())
        .args(["rules"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let pretty = String::from_utf8(output).expect("rules output should be utf-8");

    assert!(pretty.contains("cleanarch/*"));
    assert!(pretty.contains("verticalslice/*"));
    assert!(pretty.contains("frontend/*"));
    assert!(pretty.contains("cleanarch/no-layer-leak"));
    assert!(pretty.contains("default: error"));
    assert!(pretty.contains("architecture: cleanarch/*"));
    assert!(pretty.contains("aliases: onion/no-layer-leak"));
    assert!(pretty.contains("repo/test-placement"));
    assert!(pretty.contains("architecture: neutral"));
}

#[test]
fn rules_json_output_serializes_catalog_rules_with_camel_case_fields() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    let output = onioncry()
        .current_dir(workspace.path())
        .args(["rules", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let rules: Value =
        serde_json::from_slice(&output).expect("rules --format json should emit valid JSON");
    let rules = rules.as_array().expect("rules JSON should be an array");

    let layer_rule = rules
        .iter()
        .find(|rule| rule["name"] == "cleanarch/no-layer-leak")
        .expect("rules JSON should include cleanarch/no-layer-leak");
    assert_eq!(layer_rule["defaultSeverity"], "error");
    assert_eq!(layer_rule["architectureFamily"], "cleanarch/*");
    assert_eq!(layer_rule["legacyAliases"][0], "onion/no-layer-leak");
    assert!(
        layer_rule["explanation"]
            .as_str()
            .is_some_and(|text| { text.contains("Layer rules only allow imports declared") })
    );

    let repo_rule = rules
        .iter()
        .find(|rule| rule["name"] == "repo/test-placement")
        .expect("rules JSON should include repo/test-placement");
    assert_eq!(repo_rule["defaultSeverity"], "off");
    assert_eq!(repo_rule["architectureFamily"], "neutral");
    assert_eq!(
        repo_rule["legacyAliases"]
            .as_array()
            .expect("legacyAliases should be an array")
            .len(),
        0
    );
}

#[test]
fn rules_help_describes_the_subcommand() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    onioncry()
        .current_dir(workspace.path())
        .args(["rules", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("List built-in rules")
                .and(predicate::str::contains("--format <FORMAT>")),
        );
}
