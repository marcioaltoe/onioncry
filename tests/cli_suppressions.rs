mod support;

use support::*;

const SOURCE_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

#[test]
fn inline_suppression_suppresses_next_line_for_all_source_extensions() {
    for extension in SOURCE_EXTENSIONS {
        let workspace = TempDir::new().expect("workspace should be creatable");
        write_source_layer_config(&workspace.path().join(".onioncryrc.jsonc"), r#"{}"#, "[]");
        let source = if *extension == "cjs" {
            r#"// onioncry-disable-next-line cleanarch/no-layer-leak -- legacy module boundary
const { repo } = require("../infra/repo");
module.exports = { run: repo };
"#
        } else {
            r#"// onioncry-disable-next-line cleanarch/no-layer-leak -- legacy module boundary
import { repo } from "../infra/repo";
export const run = repo;
"#
        };
        let target = if *extension == "cjs" {
            "exports.repo = 1;\n"
        } else {
            "export const repo = 1;\n"
        };
        write_file(
            &workspace
                .path()
                .join(format!("src/application/use-case.{extension}")),
            source,
        );
        write_file(
            &workspace.path().join(format!("src/infra/repo.{extension}")),
            target,
        );

        let result = run_json_check(&workspace, &["check", "--format", "json"]);

        assert_eq!(
            result["summary"]["suppressedCount"], 1,
            "expected suppressed count for .{extension}"
        );
        assert_eq!(
            result["violations"][0]["suppressed"], true,
            "expected suppressed violation for .{extension}"
        );
        assert_eq!(
            result["violations"][0]["suppressionReason"],
            "legacy module boundary"
        );
    }
}

#[test]
fn inline_suppression_without_reason_reports_invalid_suppression() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_layer_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        r#"// onioncry-disable-next-line cleanarch/no-layer-leak
import { repo } from "../infra/repo";
export const run = repo;
"#,
    );
    write_file(
        &workspace.path().join("src/infra/repo.ts"),
        "export const repo = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);

    let invalid = violation_with_rule(&result, "repo/invalid-suppression");
    assert_eq!(invalid["severity"], "error");
    assert!(
        invalid["message"]
            .as_str()
            .expect("invalid suppression message should be text")
            .contains("-- <reason>")
    );
    assert_eq!(invalid["line"], 1);
    assert_eq!(
        violation_with_rule(&result, "cleanarch/no-layer-leak").get("suppressed"),
        None
    );
}

#[test]
fn inline_suppression_with_unknown_rule_reports_closest_known_rules() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_layer_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        r#"// onioncry-disable-next-line cleanarch/no-layer-leka -- typo while migrating
import { repo } from "../infra/repo";
export const run = repo;
"#,
    );
    write_file(
        &workspace.path().join("src/infra/repo.ts"),
        "export const repo = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);

    let invalid = violation_with_rule(&result, "repo/invalid-suppression");
    assert!(
        invalid["message"]
            .as_str()
            .expect("invalid suppression message should be text")
            .contains("cleanarch/no-layer-leka")
    );
    assert!(
        invalid["message"]
            .as_str()
            .expect("invalid suppression message should be text")
            .contains("cleanarch/no-layer-leak")
    );
}

#[test]
fn inline_suppression_supports_multiple_rules_and_legacy_aliases() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_layer_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        r#"// onioncry-disable-next-line onion/no-layer-leak, cleanarch/unclassified-file -- legacy boundary with stale extra rule
import { repo } from "../infra/repo";
export const run = repo;
"#,
    );
    write_file(
        &workspace.path().join("src/infra/repo.ts"),
        "export const repo = 1;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    let suppressed = violation_with_rule(&result, "cleanarch/no-layer-leak");
    assert_eq!(suppressed["suppressed"], true);
    assert_eq!(
        suppressed["suppressionReason"],
        "legacy boundary with stale extra rule"
    );
    let unused = violation_with_rule(&result, "repo/unused-suppression");
    assert_eq!(unused["severity"], "warn");
    assert!(
        unused["message"]
            .as_str()
            .expect("unused suppression message should be text")
            .contains("cleanarch/unclassified-file")
    );
}

#[test]
fn llm_output_groups_suppressed_findings_separately() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_layer_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        r#"// onioncry-disable-next-line cleanarch/no-layer-leak -- accepted legacy import
import { repo } from "../infra/repo";
export const run = repo;
"#,
    );
    write_file(
        &workspace.path().join("src/infra/repo.ts"),
        "export const repo = 1;\n",
    );

    let output = onioncry()
        .current_dir(workspace.path())
        .args(["check", "--llm-mode"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let llm = String::from_utf8(output).expect("llm output should be utf-8");

    assert!(llm.contains("problemCount: 0"));
    assert!(llm.contains("suppressedCount: 1"));
    assert!(llm.contains("state: suppressed"));
    assert!(llm.contains("actionable: false"));
    assert!(llm.contains("suppressionReason: accepted legacy import"));
}

#[test]
fn inline_suppression_rule_severity_can_be_configured_and_overridden() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_source_layer_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#"{ "repo/unused-suppression": "error" }"#,
        "[]",
    );
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        r#"// onioncry-disable-next-line cleanarch/no-layer-leak -- stale exception
export const run = 1;
"#,
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let unused = violation_with_rule(&result, "repo/unused-suppression");
    assert_eq!(unused["severity"], "error");

    let override_workspace = TempDir::new().expect("workspace should be creatable");
    write_source_layer_config(
        &override_workspace.path().join(".onioncryrc.jsonc"),
        r#"{}"#,
        r#"[
    {
      "files": ["src/application/use-case.ts"],
      "rules": {
        "repo/invalid-suppression": "warn"
      }
    }
  ]"#,
    );
    write_file(
        &override_workspace
            .path()
            .join("src/application/use-case.ts"),
        r#"// onioncry-disable-next-line cleanarch/no-layer-leak
export const run = 1;
"#,
    );

    let override_result = run_json_check(&override_workspace, &["check", "--format", "json"]);
    let invalid = violation_with_rule(&override_result, "repo/invalid-suppression");
    assert_eq!(invalid["severity"], "warn");
}

#[test]
fn rules_output_lists_inline_suppression_rules() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    let output = onioncry()
        .current_dir(workspace.path())
        .args(["rules", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let rules: Value = serde_json::from_slice(&output).expect("rules should emit JSON");

    assert!(catalog_contains(
        &rules,
        "repo/invalid-suppression",
        "error"
    ));
    assert!(catalog_contains(&rules, "repo/unused-suppression", "warn"));
}

fn write_source_layer_config(path: &std::path::Path, rules_json: &str, overrides_json: &str) {
    write_file(
        path,
        &format!(
            r#"{{
  "version": 1,
  "project": {{
    "root": ".",
    "include": ["src/**/*"],
    "exclude": []
  }},
  "aliases": {{}},
  "layers": {{
    "application": {{
      "patterns": ["src/application/**"],
      "mayImport": ["application"]
    }},
    "infra": {{
      "patterns": ["src/infra/**"],
      "mayImport": ["infra", "application"]
    }}
  }},
  "rules": {rules_json},
  "overrides": {overrides_json}
}}"#
        ),
    );
}

fn violation_with_rule<'a>(result: &'a Value, rule: &str) -> &'a Value {
    result["violations"]
        .as_array()
        .expect("violations should be an array")
        .iter()
        .find(|violation| violation["rule"] == rule)
        .unwrap_or_else(|| panic!("expected violation with rule {rule}"))
}

fn catalog_contains(rules: &Value, name: &str, default_severity: &str) -> bool {
    rules
        .as_array()
        .expect("rules should be an array")
        .iter()
        .any(|rule| rule["name"] == name && rule["defaultSeverity"] == default_severity)
}
