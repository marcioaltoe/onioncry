use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn onioncry() -> Command {
    Command::cargo_bin("onioncry").expect("onioncry binary should be built for tests")
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test parent directory should be creatable");
    }
    fs::write(path, contents).expect("test file should be writable");
}

fn strip_full_line_jsonc_comments(contents: &str) -> String {
    contents
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn write_minimal_config(path: &Path, root: &str, include: &[&str], exclude: &[&str]) {
    write_config(path, root, include, exclude, "{}");
}

fn write_config(path: &Path, root: &str, include: &[&str], exclude: &[&str], aliases_json: &str) {
    let include_json =
        serde_json::to_string(&include).expect("include patterns should serialize to json");
    let exclude_json =
        serde_json::to_string(&exclude).expect("exclude patterns should serialize to json");
    write_file(
        path,
        &format!(
            r#"{{
  // JSONC comments are part of the public configuration contract.
  "version": 1,
  "project": {{
    "root": "{root}",
    "include": {include_json},
    "exclude": {exclude_json},
  }},
  "aliases": {aliases_json},
  "rules": {{}},
  "overrides": []
}}"#
        ),
    );
}

fn write_layer_config(path: &Path) {
    write_file(
        path,
        r#"{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["src/**/*.ts"],
    "exclude": []
  },
  "aliases": {},
  "layers": {
    "domain": {
      "patterns": ["src/domain/**"],
      "mayImport": ["domain", "shared"]
    },
    "application": {
      "patterns": ["src/application/**"],
      "mayImport": ["application", "domain", "shared"]
    },
    "infra": {
      "patterns": ["src/infra/**"],
      "mayImport": ["infra", "application", "domain", "shared"]
    },
    "shared": {
      "patterns": ["src/shared/**"],
      "mayImport": ["shared"]
    }
  },
  "rules": {
    "onion/no-layer-leak": "error",
    "onion/unclassified-file": "warn"
  },
  "overrides": []
}"#,
    );
}

fn write_rule_policy_config(path: &Path) {
    write_file(
        path,
        r#"{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["src/**/*.ts"],
    "exclude": ["src/excluded/**"]
  },
  "layers": {
    "domain": {
      "patterns": ["src/domain/**"],
      "mayImport": ["domain"]
    },
    "application": {
      "patterns": ["src/application/**"],
      "mayImport": ["application", "domain"]
    }
  },
  "rules": {
    "onion/no-layer-leak": ["error", { "note": "base policy" }],
    "onion/unclassified-file": "off"
  },
  "overrides": [
    {
      "files": ["src/domain/legacy.ts"],
      "rules": {
        "onion/no-layer-leak": "off"
      }
    },
    {
      "files": ["src/domain/strict.ts"],
      "rules": {
        "onion/no-layer-leak": "off"
      }
    },
    {
      "files": ["src/domain/strict.ts"],
      "rules": {
        "onion/no-layer-leak": "warn"
      }
    },
    {
      "files": ["src/excluded/**"],
      "rules": {
        "onion/no-layer-leak": "error",
        "onion/unclassified-file": "error"
      }
    }
  ]
}"#,
    );
}

fn write_external_package_policy_config(path: &Path) {
    write_file(
        path,
        r#"{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["src/**/*.ts"],
    "exclude": []
  },
  "aliases": {
    "@domain/": "src/domain/"
  },
  "layers": {
    "domain": {
      "patterns": ["src/domain/**"],
      "mayImport": ["domain"]
    },
    "application": {
      "patterns": ["src/application/**"],
      "mayImport": ["application", "domain"]
    },
    "infra": {
      "patterns": ["src/infra/**"],
      "mayImport": ["infra", "application", "domain"]
    }
  },
  "rules": {
    "onion/no-layer-leak": "error",
    "onion/unclassified-file": "off",
    "onion/no-forbidden-imports": ["error", {
      "layers": [
        {
          "fromLayer": "domain",
          "severity": "error",
          "allow": ["uuid", "@safe/*"]
        },
        {
          "fromLayer": "application",
          "severity": "warn",
          "allow": ["@aws-sdk/*"]
        },
        {
          "fromLayer": "infra",
          "severity": "off",
          "allow": []
        }
      ]
    }]
  },
  "overrides": []
}"#,
    );
}

fn write_context_policy_config(path: &Path) {
    write_file(
        path,
        r#"{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["src/**/*.ts"],
    "exclude": []
  },
  "contexts": {
    "sales": {
      "patterns": ["src/sales/**", "src/ambiguous/**"]
    },
    "billing": {
      "patterns": ["src/billing/**", "src/ambiguous/**"]
    }
  },
  "contextRules": {
    "default": {
      "allowSameContext": true,
      "allowCrossContext": ["contracts", "events", "ports", "shared"]
    }
  },
  "rules": {
    "onion/no-cross-context-internal-import": "error"
  },
  "overrides": []
}"#,
    );
}

fn write_cycle_policy_config(path: &Path) {
    write_file(
        path,
        r#"{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["src/**/*.ts"],
    "exclude": []
  },
  "rules": {
    "onion/circular-dependency": "warn",
    "onion/unresolved-import": "off"
  },
  "overrides": [
    {
      "files": ["src/suppressed/**"],
      "rules": {
        "onion/circular-dependency": "off"
      }
    }
  ]
}"#,
    );
}

fn write_explain_config(path: &Path) {
    write_file(
        path,
        r#"{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["src/**/*.ts"],
    "exclude": []
  },
  "layers": {
    "domain": {
      "patterns": ["src/**/domain/**", "src/billing/internal/**"],
      "mayImport": ["domain", "shared"]
    },
    "application": {
      "patterns": ["src/**/application/**"],
      "mayImport": ["application", "domain", "shared"]
    },
    "shared": {
      "patterns": ["src/shared/**"],
      "mayImport": ["shared"]
    }
  },
  "contexts": {
    "sales": {
      "patterns": ["src/sales/**"]
    },
    "billing": {
      "patterns": ["src/billing/**"]
    }
  },
  "contextRules": {
    "default": {
      "allowSameContext": true,
      "allowCrossContext": ["contracts", "events", "ports", "shared"]
    }
  },
  "rules": {
    "onion/no-layer-leak": "error",
    "onion/no-cross-context-internal-import": "error",
    "onion/no-forbidden-imports": ["error", {
      "layers": [
        {
          "fromLayer": "domain",
          "severity": "error",
          "allow": ["uuid"]
        },
        {
          "fromLayer": "application",
          "severity": "warn",
          "allow": []
        }
      ]
    }],
    "onion/unresolved-import": "warn",
    "onion/unclassified-file": "warn"
  },
  "overrides": []
}"#,
    );
}

fn run_json_check(workspace: &TempDir, args: &[&str]) -> Value {
    let output = onioncry()
        .current_dir(workspace.path())
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).expect("check --format json should emit valid JSON")
}

fn run_json_explain(workspace: &TempDir, args: &[&str]) -> Value {
    let output = onioncry()
        .current_dir(workspace.path())
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).expect("explain --format json should emit valid JSON")
}

fn run_json_check_failure(workspace: &TempDir, args: &[&str]) -> Value {
    let output = onioncry()
        .current_dir(workspace.path())
        .args(args)
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).expect("failing check --format json should emit valid JSON")
}

#[test]
fn init_creates_parseable_mvp_template() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    onioncry()
        .current_dir(workspace.path())
        .args(["init"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".onioncryrc.jsonc"));

    let config_path = workspace.path().join(".onioncryrc.jsonc");
    let config = fs::read_to_string(&config_path).expect("init config should be readable");

    assert!(config.contains(r#""$schema""#));
    assert!(config.contains(r#""version""#));
    assert!(config.contains(r#""project""#));
    assert!(config.contains(r#""aliases""#));
    assert!(config.contains(r#""layers""#));
    assert!(config.contains(r#""contexts""#));
    assert!(config.contains(r#""contextRules""#));
    assert!(config.contains(r#""rules""#));
    assert!(config.contains(r#""overrides""#));
    assert!(config.contains("TODO"));
    assert!(config.contains(r#""domain""#));
    assert!(config.contains(r#""application""#));
    assert!(config.contains(r#""infra""#));
    assert!(config.contains(r#""shared""#));
    assert!(config.contains(r#""mayImport": ["domain", "shared"]"#));
    assert!(config.contains(r#""onion/no-layer-leak": "error""#));
    assert!(config.contains(r#""onion/no-cross-context-internal-import": "error""#));
    assert!(config.contains(r#""fromLayer": "domain""#));
    assert!(config.contains(r#""severity": "error""#));
    assert!(config.contains(r#""fromLayer": "application""#));
    assert!(config.contains(r#""severity": "warn""#));
    assert!(config.contains(r#""fromLayer": "infra""#));
    assert!(config.contains(r#""severity": "off""#));
    assert!(config.contains(r#""onion/unresolved-import": "warn""#));
    assert!(config.contains(r#""onion/circular-dependency": "warn""#));
    assert!(config.contains(r#""onion/unclassified-file": "warn""#));

    let stripped = strip_full_line_jsonc_comments(&config);
    let parsed: Value =
        serde_json::from_str(&stripped).expect("template should parse after comments are stripped");
    assert_eq!(parsed["version"], 1);
    assert!(parsed["project"].is_object());
    assert!(parsed["aliases"].is_object());
    assert!(parsed["layers"].is_object());
    assert!(parsed["contexts"].is_object());
    assert!(parsed["contextRules"].is_object());
    assert!(parsed["rules"].is_object());
    assert!(parsed["overrides"].is_array());
}

#[test]
fn init_does_not_overwrite_existing_config_unless_forced() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    let config_path = workspace.path().join(".onioncryrc.jsonc");
    write_file(&config_path, "existing config\n");

    onioncry()
        .current_dir(workspace.path())
        .args(["init"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains(".onioncryrc.jsonc").and(predicate::str::contains("--force")),
        );
    assert_eq!(
        fs::read_to_string(&config_path).expect("existing config should be readable"),
        "existing config\n"
    );

    onioncry()
        .current_dir(workspace.path())
        .args(["init", "--force"])
        .assert()
        .success();

    let forced = fs::read_to_string(&config_path).expect("forced config should be readable");
    assert!(forced.contains(r#""onion/no-layer-leak": "error""#));
    assert_ne!(forced, "existing config\n");
}

#[test]
fn check_discovers_default_jsonc_config_and_emits_json_result() {
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

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["fileCount"], 1);
    assert_eq!(result["summary"]["errorCount"], 0);
    assert_eq!(result["summary"]["warningCount"], 0);
    assert_eq!(
        result["violations"]
            .as_array()
            .expect("violations should be an array")
            .len(),
        0
    );
}

#[test]
fn check_accepts_explicit_config_path() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_minimal_config(
        &workspace.path().join("config/onioncry.jsonc"),
        "../project",
        &["**/*.ts"],
        &[],
    );
    write_file(
        &workspace.path().join("project/application/use-case.ts"),
        "export const run = () => undefined;\n",
    );

    let result = run_json_check(
        &workspace,
        &[
            "check",
            "--config",
            "config/onioncry.jsonc",
            "--format",
            "json",
        ],
    );

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["fileCount"], 1);
}

#[test]
fn check_reports_missing_default_config() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    onioncry()
        .current_dir(workspace.path())
        .args(["check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".onioncryrc.jsonc"));
}

#[test]
fn check_applies_include_and_exclude_before_reporting_summary() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_minimal_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        &["src/**/*.{ts,tsx}"],
        &["src/**/__tests__/**", "src/**/*.spec.ts"],
    );
    write_file(
        &workspace.path().join("src/domain/order.ts"),
        "export const order = 1;\n",
    );
    write_file(
        &workspace.path().join("src/domain/order.spec.ts"),
        "export const testOrder = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/__tests__/use-case.ts"),
        "export const testUseCase = 1;\n",
    );
    write_file(&workspace.path().join("README.md"), "# ignored\n");

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["summary"]["fileCount"], 1);
}

#[test]
fn check_accepts_warning_failure_threshold_with_empty_violations() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_minimal_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        &["src/**/*.ts"],
        &[],
    );
    write_file(
        &workspace.path().join("src/shared/id.ts"),
        "export const id = 1;\n",
    );

    onioncry()
        .current_dir(workspace.path())
        .args(["check", "--fail-on", "warning"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status: pass"));
}

#[test]
fn check_reports_unresolved_local_imports_with_source_locations() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_minimal_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        &["src/**/*.ts"],
        &[],
    );
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        r#"import value from "./missing-static";
import type { Thing } from "./missing-type";
export { Other } from "./missing-reexport";
const lazy = import("./missing-dynamic");
const common = require("./missing-require");
import "react";
const ignoredDynamic = import(`./${name}`);
const ignoredRequire = require(name);
"#,
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["warningCount"], 5);
    assert_eq!(violations.len(), 5);

    let specifiers = violations
        .iter()
        .map(|violation| violation["importSpecifier"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(
        specifiers,
        vec![
            "./missing-static",
            "./missing-type",
            "./missing-reexport",
            "./missing-dynamic",
            "./missing-require"
        ]
    );
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "onion/unresolved-import"
            && violation["severity"] == "warn"
            && violation["line"].as_u64().is_some_and(|line| line > 0)
            && violation["column"]
                .as_u64()
                .is_some_and(|column| column > 0)
            && violation["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("src/application/use-case.ts"))
    }));
}

#[test]
fn check_resolves_relative_alias_extension_and_index_imports() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        &["src/**/*.{ts,tsx,js}"],
        &[],
        r#"{ "@app/": "src/" }"#,
    );
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        r#"import type { Thing } from "./types";
export * from "./contracts";
const lazy = import("./dynamic");
const common = require("@app/infra/repo");
"#,
    );
    write_file(
        &workspace.path().join("src/application/types.ts"),
        "export type Thing = {};\n",
    );
    write_file(
        &workspace.path().join("src/application/contracts/index.ts"),
        "export const contract = 1;\n",
    );
    write_file(
        &workspace.path().join("src/application/dynamic.tsx"),
        "export const lazy = 1;\n",
    );
    write_file(
        &workspace.path().join("src/infra/repo/index.js"),
        "module.exports = {};\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["warningCount"], 0);
    assert_eq!(
        result["violations"]
            .as_array()
            .expect("violations should be an array")
            .len(),
        0
    );
}

#[test]
fn check_allows_configured_layer_imports() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_layer_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        r#"import { Order } from "../domain/order";
import { Id } from "../shared/id";
export const run = (order: Order, id: Id) => ({ order, id });
"#,
    );
    write_file(
        &workspace.path().join("src/domain/order.ts"),
        "export type Order = { id: string };\n",
    );
    write_file(
        &workspace.path().join("src/shared/id.ts"),
        "export type Id = string;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["errorCount"], 0);
    assert_eq!(result["summary"]["warningCount"], 0);
}

#[test]
fn check_reports_layer_leaks_for_type_only_imports_and_reexports() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_layer_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/domain/order.ts"),
        r#"import type { UseCase } from "../application/use-case";
export { run } from "../application/use-case";
export type Order = { useCase?: UseCase };
"#,
    );
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        "export type UseCase = () => void;\nexport const run = () => undefined;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["errorCount"], 2);
    assert_eq!(violations.len(), 2);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "onion/no-layer-leak"
            && violation["severity"] == "error"
            && violation["fromLayer"] == "domain"
            && violation["toLayer"] == "application"
            && violation["importSpecifier"] == "../application/use-case"
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("mayImport"))
    }));
}

#[test]
fn check_reports_ambiguous_and_unclassified_layer_files() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_file(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#"{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["src/**/*.ts"],
    "exclude": []
  },
  "layers": {
    "domain": {
      "patterns": ["src/domain/**", "src/ambiguous/**"],
      "mayImport": ["domain"]
    },
    "shared": {
      "patterns": ["src/shared/**", "src/ambiguous/**"],
      "mayImport": ["shared"]
    }
  },
  "rules": {
    "onion/unclassified-file": "warn"
  },
  "overrides": []
}"#,
    );
    write_file(
        &workspace.path().join("src/ambiguous/value.ts"),
        "export const value = 1;\n",
    );
    write_file(
        &workspace.path().join("src/other/value.ts"),
        "export const value = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "onion/ambiguous-layer"
            && violation["severity"] == "error"
            && violation["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("src/ambiguous/value.ts"))
            && violation["matchedLayers"]
                .as_array()
                .is_some_and(|layers| layers.len() == 2)
    }));
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "onion/unclassified-file"
            && violation["severity"] == "warn"
            && violation["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("src/other/value.ts"))
    }));
}

#[test]
fn check_applies_linter_style_rule_policy_and_warning_threshold() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_rule_policy_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/domain/legacy.ts"),
        r#"import { run } from "../application/use-case";
export const legacy = run;
"#,
    );
    write_file(
        &workspace.path().join("src/domain/strict.ts"),
        r#"import { run } from "../application/use-case";
export const strict = run;
"#,
    );
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        "export const run = () => undefined;\n",
    );
    write_file(
        &workspace.path().join("src/other/unclassified.ts"),
        "export const value = 1;\n",
    );
    write_file(
        &workspace.path().join("src/excluded/ignored.ts"),
        r#"import { run } from "../application/use-case";
export const ignored = run;
"#,
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["fileCount"], 4);
    assert_eq!(result["summary"]["warningCount"], 1);
    assert_eq!(result["summary"]["errorCount"], 0);
    assert_eq!(result["summary"]["violationCount"], 1);
    assert_eq!(result["violations"][0]["rule"], "onion/no-layer-leak");
    assert_eq!(result["violations"][0]["severity"], "warn");
    assert!(
        result["violations"][0]["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("src/domain/strict.ts"))
    );

    let warning_failure = run_json_check_failure(
        &workspace,
        &["check", "--format", "json", "--fail-on", "warning"],
    );

    assert_eq!(warning_failure["status"], "fail");
    assert_eq!(warning_failure["summary"]["warningCount"], 1);
    assert_eq!(warning_failure["summary"]["errorCount"], 0);
}

#[test]
fn check_rejects_unknown_rules_and_invalid_severities() {
    let unknown_rule_workspace = TempDir::new().expect("workspace should be creatable");
    write_file(
        &unknown_rule_workspace.path().join(".onioncryrc.jsonc"),
        r#"{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["src/**/*.ts"],
    "exclude": []
  },
  "rules": {
    "onion/not-a-rule": "warn"
  },
  "overrides": []
}"#,
    );
    write_file(
        &unknown_rule_workspace.path().join("src/domain/order.ts"),
        "export const order = 1;\n",
    );

    onioncry()
        .current_dir(unknown_rule_workspace.path())
        .args(["check"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("unknown rule")
                .and(predicate::str::contains("onion/not-a-rule")),
        );

    let invalid_severity_workspace = TempDir::new().expect("workspace should be creatable");
    write_file(
        &invalid_severity_workspace.path().join(".onioncryrc.jsonc"),
        r#"{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["src/**/*.ts"],
    "exclude": []
  },
  "rules": {
    "onion/no-layer-leak": "info"
  },
  "overrides": []
}"#,
    );
    write_file(
        &invalid_severity_workspace
            .path()
            .join("src/domain/order.ts"),
        "export const order = 1;\n",
    );

    onioncry()
        .current_dir(invalid_severity_workspace.path())
        .args(["check"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("invalid severity")
                .and(predicate::str::contains("onion/no-layer-leak"))
                .and(predicate::str::contains("off, warn, or error")),
        );
}

#[test]
fn check_enforces_external_package_policy_by_layer() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_external_package_policy_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/domain/order.ts"),
        r#"import { v4 } from "uuid/v4";
import { Money } from "@safe/money";
import crypto from "node:crypto";
import fs from "node:fs";
import express from "express";
import vendor from "@vendor/tool/subpath";
import { Local } from "@domain/local";
export const order = [v4, Money, crypto, fs, express, vendor, Local];
"#,
    );
    write_file(
        &workspace.path().join("src/domain/local.ts"),
        "export const Local = 1;\n",
    );
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        r#"import s3 from "@aws-sdk/client-s3";
import axios from "axios";
export const run = [s3, axios];
"#,
    );
    write_file(
        &workspace.path().join("src/infra/repo.ts"),
        r#"import pg from "pg";
import path from "node:path";
export const repo = [pg, path];
"#,
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["fileCount"], 4);
    assert_eq!(result["summary"]["errorCount"], 4);
    assert_eq!(result["summary"]["warningCount"], 1);
    assert_eq!(violations.len(), 5);

    let packages = violations
        .iter()
        .map(|violation| {
            (
                violation["severity"].as_str().unwrap_or_default(),
                violation["fromLayer"].as_str().unwrap_or_default(),
                violation["packageName"].as_str().unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();

    assert!(packages.contains(&("error", "domain", "node:crypto")));
    assert!(packages.contains(&("error", "domain", "node:fs")));
    assert!(packages.contains(&("error", "domain", "express")));
    assert!(packages.contains(&("error", "domain", "@vendor/tool")));
    assert!(packages.contains(&("warn", "application", "axios")));
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "onion/no-forbidden-imports"
            && violation["line"].as_u64().is_some_and(|line| line > 0)
            && violation["column"]
                .as_u64()
                .is_some_and(|column| column > 0)
    }));
}

#[test]
fn check_enforces_bounded_context_public_surface() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_context_policy_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/sales/use-case.ts"),
        r#"import { local } from "./internal/order";
import { BillingApi } from "../billing/contracts/api";
import { BillingEvent } from "../billing/events/created";
import { BillingPort } from "../billing/ports/repository";
import { BillingShared } from "../billing/shared/money";
import { secret } from "../billing/internal/model";
import { id } from "../shared/id";
export const run = [local, BillingApi, BillingEvent, BillingPort, BillingShared, secret, id];
"#,
    );
    write_file(
        &workspace.path().join("src/sales/internal/order.ts"),
        "export const local = 1;\n",
    );
    write_file(
        &workspace.path().join("src/billing/contracts/api.ts"),
        "export const BillingApi = 1;\n",
    );
    write_file(
        &workspace.path().join("src/billing/events/created.ts"),
        "export const BillingEvent = 1;\n",
    );
    write_file(
        &workspace.path().join("src/billing/ports/repository.ts"),
        "export const BillingPort = 1;\n",
    );
    write_file(
        &workspace.path().join("src/billing/shared/money.ts"),
        "export const BillingShared = 1;\n",
    );
    write_file(
        &workspace.path().join("src/billing/internal/model.ts"),
        "export const secret = 1;\n",
    );
    write_file(
        &workspace.path().join("src/shared/id.ts"),
        "export const id = 1;\n",
    );
    write_file(
        &workspace.path().join("src/tools/contextless.ts"),
        r#"import { secret } from "../billing/internal/model";
export const tool = secret;
"#,
    );
    write_file(
        &workspace.path().join("src/ambiguous/value.ts"),
        "export const value = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["fileCount"], 10);
    assert_eq!(result["summary"]["errorCount"], 2);
    assert_eq!(result["summary"]["warningCount"], 0);
    assert_eq!(violations.len(), 2);

    assert!(violations.iter().any(|violation| {
        violation["rule"] == "onion/no-cross-context-internal-import"
            && violation["severity"] == "error"
            && violation["fromContext"] == "sales"
            && violation["toContext"] == "billing"
            && violation["importSpecifier"] == "../billing/internal/model"
            && violation["targetFile"]
                .as_str()
                .is_some_and(|file| file.ends_with("src/billing/internal/model.ts"))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("public surface"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "onion/ambiguous-context"
            && violation["severity"] == "error"
            && violation["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("src/ambiguous/value.ts"))
            && violation["matchedContexts"]
                .as_array()
                .is_some_and(|contexts| contexts.len() == 2)
    }));
}

#[test]
fn check_detects_circular_dependencies_and_honors_rule_policy() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_cycle_policy_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/simple/a.ts"),
        r#"import { b } from "./b";
export const a = b;
"#,
    );
    write_file(
        &workspace.path().join("src/simple/b.ts"),
        r#"import { a } from "./a";
export const b = a;
"#,
    );
    write_file(
        &workspace.path().join("src/long/a.ts"),
        r#"import { b } from "./b";
export const a = b;
"#,
    );
    write_file(
        &workspace.path().join("src/long/b.ts"),
        r#"import { c } from "./c";
export const b = c;
"#,
    );
    write_file(
        &workspace.path().join("src/long/c.ts"),
        r#"import { a } from "./a";
export const c = a;
"#,
    );
    write_file(
        &workspace.path().join("src/acyclic/a.ts"),
        r#"import { b } from "./b";
export const a = b;
"#,
    );
    write_file(
        &workspace.path().join("src/acyclic/b.ts"),
        "export const b = 1;\n",
    );
    write_file(
        &workspace.path().join("src/ignored/external.ts"),
        r#"import react from "react";
export const external = react;
"#,
    );
    write_file(
        &workspace.path().join("src/ignored/unresolved.ts"),
        r#"import { missing } from "./missing";
export const unresolved = missing;
"#,
    );
    write_file(
        &workspace.path().join("src/suppressed/a.ts"),
        r#"import { b } from "./b";
export const a = b;
"#,
    );
    write_file(
        &workspace.path().join("src/suppressed/b.ts"),
        r#"import { a } from "./a";
export const b = a;
"#,
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["fileCount"], 11);
    assert_eq!(result["summary"]["warningCount"], 2);
    assert_eq!(result["summary"]["errorCount"], 0);
    assert_eq!(violations.len(), 2);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "onion/circular-dependency"
            && violation["severity"] == "warn"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains(" -> "))
    }));

    let mut cycle_paths = violations
        .iter()
        .map(|violation| {
            violation["cyclePath"]
                .as_array()
                .expect("cyclePath should be an array")
                .iter()
                .map(|path| {
                    path.as_str()
                        .expect("cycle path entries should be strings")
                        .to_string()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    cycle_paths.sort();

    assert_eq!(
        cycle_paths,
        vec![
            vec![
                "src/long/a.ts".to_string(),
                "src/long/b.ts".to_string(),
                "src/long/c.ts".to_string(),
                "src/long/a.ts".to_string(),
            ],
            vec![
                "src/simple/a.ts".to_string(),
                "src/simple/b.ts".to_string(),
                "src/simple/a.ts".to_string(),
            ],
        ]
    );

    let warning_failure = run_json_check_failure(
        &workspace,
        &["check", "--format", "json", "--fail-on", "warning"],
    );

    assert_eq!(warning_failure["status"], "fail");
    assert_eq!(warning_failure["summary"]["warningCount"], 2);
    assert_eq!(warning_failure["summary"]["errorCount"], 0);
}

#[test]
fn explain_reports_classification_imports_package_policy_and_violations() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_explain_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/sales/domain/order.ts"),
        r#"import { useCase } from "../application/use-case";
import { BillingApi } from "../../billing/contracts/api";
import { secret } from "../../billing/internal/model";
import { id } from "../../shared/id";
import { v4 } from "uuid/v4";
import express from "express";
import { missing } from "./missing";
export const order = [useCase, BillingApi, secret, id, v4, express, missing];
"#,
    );
    write_file(
        &workspace.path().join("src/sales/application/use-case.ts"),
        "export const useCase = 1;\n",
    );
    write_file(
        &workspace.path().join("src/billing/contracts/api.ts"),
        "export const BillingApi = 1;\n",
    );
    write_file(
        &workspace.path().join("src/billing/internal/model.ts"),
        "export const secret = 1;\n",
    );
    write_file(
        &workspace.path().join("src/shared/id.ts"),
        "export const id = 1;\n",
    );

    let result = run_json_explain(
        &workspace,
        &["explain", "src/sales/domain/order.ts", "--format", "json"],
    );

    assert!(
        result["file"]
            .as_str()
            .unwrap_or_default()
            .ends_with("src/sales/domain/order.ts")
    );
    assert_eq!(result["layer"]["status"], "classified");
    assert_eq!(result["layer"]["name"], "domain");
    assert_eq!(result["layer"]["matchedPatterns"][0], "src/**/domain/**");
    assert_eq!(result["context"]["status"], "classified");
    assert_eq!(result["context"]["name"], "sales");
    assert_eq!(result["publicSurface"], false);

    let imports = result["imports"]
        .as_array()
        .expect("imports should be an array");
    assert_eq!(imports.len(), 7);
    assert!(imports.iter().any(|import| {
        import["specifier"] == "../application/use-case"
            && import["resolution"] == "local"
            && import["targetFile"]
                .as_str()
                .is_some_and(|file| file.ends_with("src/sales/application/use-case.ts"))
    }));
    assert!(imports.iter().any(|import| {
        import["specifier"] == "uuid/v4"
            && import["resolution"] == "external"
            && import["packageName"] == "uuid"
            && import["packageAllowed"] == true
    }));
    assert!(imports.iter().any(|import| {
        import["specifier"] == "express"
            && import["resolution"] == "external"
            && import["packageName"] == "express"
            && import["packageAllowed"] == false
    }));
    assert!(imports.iter().any(|import| {
        import["specifier"] == "./missing" && import["resolution"] == "unresolvedLocal"
    }));

    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");
    let rules = violations
        .iter()
        .map(|violation| violation["rule"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();

    assert!(rules.contains(&"onion/no-layer-leak"));
    assert!(rules.contains(&"onion/no-cross-context-internal-import"));
    assert!(rules.contains(&"onion/no-forbidden-imports"));
    assert!(rules.contains(&"onion/unresolved-import"));
    assert!(
        violations
            .iter()
            .all(|violation| violation["suggestion"].is_string()
                || violation["suggestion"].is_null())
    );
}

#[test]
fn explain_reports_unclassified_and_contextless_files_without_failing() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_explain_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/tools/script.ts"),
        r#"import react from "react";
export const script = react;
"#,
    );

    let result = run_json_explain(
        &workspace,
        &["explain", "src/tools/script.ts", "--format", "json"],
    );

    assert_eq!(result["layer"]["status"], "unclassified");
    assert!(result["layer"]["name"].is_null());
    assert_eq!(result["context"]["status"], "contextless");
    assert!(result["context"]["name"].is_null());
    assert_eq!(result["publicSurface"], false);
    assert_eq!(result["imports"][0]["resolution"], "external");
    assert_eq!(result["imports"][0]["packageName"], "react");
    assert!(result["imports"][0]["packageAllowed"].is_null());
    assert_eq!(result["violations"][0]["rule"], "onion/unclassified-file");
}
