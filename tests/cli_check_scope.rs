mod support;

use support::*;

fn write_two_violating_layers(workspace: &TempDir) {
    write_layer_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/domain/user.ts"),
        "import { db } from \"../infra/db\";\nexport const user = db;\n",
    );
    write_file(
        &workspace.path().join("src/application/service.ts"),
        "import { db } from \"../infra/db\";\nexport const service = db;\n",
    );
    write_file(
        &workspace.path().join("src/infra/db.ts"),
        "export const db = {};\n",
    );
}

#[test]
fn check_files_scopes_the_report_while_analysis_stays_whole_project() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_two_violating_layers(&workspace);

    let output = onioncry()
        .current_dir(workspace.path())
        .args(["check", "--files", "src/domain/user.ts", "--format", "json"])
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&output).expect("check should emit JSON");

    assert_eq!(report["status"], "fail");
    assert_eq!(report["summary"]["violationCount"], 1);
    assert_eq!(report["summary"]["errorCount"], 1);
    assert!(
        report["violations"][0]["file"]
            .as_str()
            .expect("violation file should be a string")
            .ends_with("src/domain/user.ts")
    );

    onioncry()
        .current_dir(workspace.path())
        .args(["check", "--files", "src/infra/db.ts"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status: pass"));
}

#[test]
fn check_files_skips_paths_outside_the_file_universe_without_failing() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_two_violating_layers(&workspace);

    onioncry()
        .current_dir(workspace.path())
        .args(["check", "--files", "src/missing.ts", "src/domain/user.ts"])
        .assert()
        .code(1)
        .stderr(
            predicate::str::contains("not in the analyzed file universe")
                .and(predicate::str::contains("src/missing.ts")),
        );

    onioncry()
        .current_dir(workspace.path())
        .args(["check", "--files", "src/missing.ts"])
        .assert()
        .success()
        .stderr(predicate::str::contains("src/missing.ts"));
}

#[test]
fn check_files_always_reports_project_level_context_cycles() {
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
  "contexts": {
    "sales": { "patterns": ["src/sales/**"] },
    "billing": { "patterns": ["src/billing/**"] }
  },
  "contextRules": {
    "default": {
      "allowSameContext": true,
      "allowCrossContext": ["contracts"]
    }
  },
  "rules": {
    "cleanarch/no-cross-context-internal-import": "error",
    "cleanarch/no-context-cycle": "error"
  },
  "overrides": []
}"#,
    );
    write_file(
        &workspace.path().join("src/sales/a.ts"),
        "import { b } from \"../billing/b\";\nexport const a = b;\n",
    );
    write_file(
        &workspace.path().join("src/billing/b.ts"),
        "import { a } from \"../sales/a\";\nexport const b = a;\n",
    );
    write_file(
        &workspace.path().join("src/tools/helper.ts"),
        "export const helper = 1;\n",
    );

    let output = onioncry()
        .current_dir(workspace.path())
        .args([
            "check",
            "--files",
            "src/tools/helper.ts",
            "--format",
            "json",
        ])
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&output).expect("check should emit JSON");
    let violations = report["violations"]
        .as_array()
        .expect("violations should be an array");

    assert!(
        violations
            .iter()
            .all(|violation| violation["rule"] == "cleanarch/no-context-cycle")
    );
    assert!(!violations.is_empty());
}

#[test]
fn check_files_conflicts_with_write_baseline() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_two_violating_layers(&workspace);

    onioncry()
        .current_dir(workspace.path())
        .args(["check", "--files", "src/domain/user.ts", "--write-baseline"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn check_files_composes_with_baseline_consumption_and_llm_mode() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_two_violating_layers(&workspace);

    onioncry()
        .current_dir(workspace.path())
        .args(["check", "--write-baseline"])
        .assert()
        .code(1);

    let output = onioncry()
        .current_dir(workspace.path())
        .args(["check", "--files", "src/domain/user.ts", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&output).expect("check should emit JSON");

    assert_eq!(report["status"], "pass");
    assert_eq!(report["summary"]["violationCount"], 0);
    assert_eq!(report["summary"]["baselinedCount"], 1);

    let llm_output = onioncry()
        .current_dir(workspace.path())
        .args([
            "check",
            "--files",
            "src/domain/user.ts",
            "--llm-mode",
            "--no-baseline",
        ])
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let llm = String::from_utf8(llm_output).expect("llm output should be utf-8");
    assert!(llm.contains("src/domain/user.ts"));
    assert!(!llm.contains("src/application/service.ts"));
}
