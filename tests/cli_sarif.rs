mod support;

use support::*;

const SARIF_SCHEMA: &str = include_str!("fixtures/sarif-schema-2.1.0.json");
const SARIF_SCHEMA_URI: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/cs01/schemas/sarif-schema-2.1.0.json";

#[test]
fn check_sarif_output_validates_against_schema_for_clean_run() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_minimal_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        &["src/**/*.ts"],
        &[],
    );
    write_file(
        &workspace.path().join("src/domain/order.ts"),
        "export const id = 1;\n",
    );

    let sarif = run_sarif_check(&workspace, true);

    assert_valid_sarif(&sarif);
    assert_eq!(sarif["$schema"], SARIF_SCHEMA_URI);
    assert_eq!(sarif["version"], "2.1.0");
    assert_eq!(sarif["runs"][0]["tool"]["driver"]["name"], "OnionCry");
    assert_eq!(
        sarif["runs"][0]["results"]
            .as_array()
            .expect("SARIF results should be an array")
            .len(),
        0
    );
}

#[test]
fn check_sarif_output_reports_failing_violation_with_rule_metadata_and_location() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_layer_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        r#"import { repo } from "../infra/repo";
export const run = repo;
"#,
    );
    write_file(
        &workspace.path().join("src/infra/repo.ts"),
        "export const repo = 1;\n",
    );

    let sarif = run_sarif_check(&workspace, false);

    assert_valid_sarif(&sarif);
    let result = &sarif["runs"][0]["results"][0];
    assert_eq!(result["ruleId"], "cleanarch/no-layer-leak");
    assert_eq!(result["level"], "error");
    assert!(
        result["message"]["text"]
            .as_str()
            .expect("message should be text")
            .contains("application may not import infra")
    );

    let location = &result["locations"][0]["physicalLocation"];
    assert!(
        location["artifactLocation"]["uri"]
            .as_str()
            .expect("SARIF uri should be a string")
            .ends_with("src/application/use-case.ts")
    );
    assert_eq!(location["region"]["startLine"], 1);
    assert!(
        location["region"]["startColumn"]
            .as_u64()
            .expect("SARIF should include a start column")
            >= 1
    );

    let rules = sarif["runs"][0]["tool"]["driver"]["rules"]
        .as_array()
        .expect("driver rules should be an array");
    let rule = rules
        .iter()
        .find(|rule| rule["id"] == "cleanarch/no-layer-leak")
        .expect("driver should include cleanarch/no-layer-leak metadata");
    assert_eq!(rule["name"], "cleanarch/no-layer-leak");
    assert!(
        rule["fullDescription"]["text"]
            .as_str()
            .expect("fullDescription should be text")
            .contains("Layer rules only allow imports")
    );
}

#[test]
fn check_sarif_output_marks_baselined_violations_as_external_suppressions() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_layer_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        r#"import { repo } from "../infra/repo";
export const run = repo;
"#,
    );
    write_file(
        &workspace.path().join("src/infra/repo.ts"),
        "export const repo = 1;\n",
    );
    write_file(
        &workspace.path().join(".onioncry-baseline.json"),
        r#"{
  "version": 1,
  "entries": [
    {
      "file": "src/application/use-case.ts",
      "rule": "cleanarch/no-layer-leak",
      "target": "../infra/repo",
      "count": 1
    }
  ]
}"#,
    );

    let sarif = run_sarif_check(&workspace, true);

    assert_valid_sarif(&sarif);
    let result = &sarif["runs"][0]["results"][0];
    assert_eq!(result["ruleId"], "cleanarch/no-layer-leak");
    assert_eq!(result["level"], "error");
    assert_eq!(result["suppressions"][0]["kind"], "external");
    assert_eq!(result["suppressions"][0]["state"], "accepted");
    assert!(
        result["suppressions"][0]["justification"]
            .as_str()
            .expect("suppression justification should be text")
            .contains("OnionCry violation baseline")
    );
}

fn run_sarif_check(workspace: &TempDir, should_pass: bool) -> Value {
    let mut assert = onioncry()
        .current_dir(workspace.path())
        .args(["check", "--format", "sarif"])
        .assert();

    assert = if should_pass {
        assert.success()
    } else {
        assert.failure()
    };

    serde_json::from_slice(&assert.get_output().stdout)
        .expect("check --format sarif should emit valid JSON")
}

fn assert_valid_sarif(sarif: &Value) {
    let schema: Value = serde_json::from_str(SARIF_SCHEMA).expect("SARIF schema should parse");
    let validator = jsonschema::validator_for(&schema).expect("SARIF schema should compile");
    let errors = validator
        .iter_errors(sarif)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();

    assert!(
        errors.is_empty(),
        "SARIF output should validate against SARIF 2.1.0 schema:\n{}",
        errors.join("\n")
    );
}
