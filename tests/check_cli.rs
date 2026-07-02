use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
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

fn git(workspace: &Path, args: &[&str]) {
    let output = ProcessCommand::new("git")
        .current_dir(workspace)
        .arg("-c")
        .arg("user.name=OnionCry Test")
        .arg("-c")
        .arg("user.email=onioncry@example.test")
        .arg("-c")
        .arg("commit.gpgsign=false")
        .args(args)
        .output()
        .expect("git should run");
    assert!(
        output.status.success(),
        "git command failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_llm_report_metadata_line(metadata: &str) {
    let revision = metadata
        .strip_prefix("onioncry-llm-report v1 revision: ")
        .expect("llm report metadata should include a revision");

    assert!(!revision.is_empty());
    assert!(!revision.contains("buildTimestamp"));
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
    "cleanarch/no-layer-leak": "error",
    "cleanarch/unclassified-file": "warn"
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
    "cleanarch/no-layer-leak": ["error", { "note": "base policy" }],
    "cleanarch/unclassified-file": "off"
  },
  "overrides": [
    {
      "files": ["src/domain/legacy.ts"],
      "rules": {
        "cleanarch/no-layer-leak": "off"
      }
    },
    {
      "files": ["src/domain/strict.ts"],
      "rules": {
        "cleanarch/no-layer-leak": "off"
      }
    },
    {
      "files": ["src/domain/strict.ts"],
      "rules": {
        "cleanarch/no-layer-leak": "warn"
      }
    },
    {
      "files": ["src/excluded/**"],
      "rules": {
        "cleanarch/no-layer-leak": "error",
        "cleanarch/unclassified-file": "error"
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
    "cleanarch/no-layer-leak": "error",
    "cleanarch/unclassified-file": "off",
    "cleanarch/no-forbidden-imports": ["error", {
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
    "cleanarch/no-cross-context-internal-import": "error"
  },
  "overrides": []
}"#,
    );
}

fn write_architecture_rules_config(path: &Path) {
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
    "application": {
      "patterns": ["src/**/application/**"],
      "mayImport": ["application", "domain", "shared"]
    },
    "domain": {
      "patterns": ["src/**/domain/**"],
      "mayImport": ["domain", "shared"]
    },
    "infra": {
      "patterns": ["src/**/infra/**"],
      "mayImport": ["infra", "application", "domain", "shared"]
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
    "cleanarch/no-layer-leak": "off",
    "cleanarch/no-forbidden-imports": "off",
    "cleanarch/unclassified-file": "off",
    "cleanarch/no-framework-in-core": "error",
    "cleanarch/no-outer-data-format-in-core": "error",
    "cleanarch/no-public-surface-internal-reexport": "error",
    "cleanarch/no-context-cycle": "error",
    "cleanarch/no-unowned-schema-import": "error",
    "solid/no-concrete-dependency": "warn",
    "codesmells/feature-envy": ["warn", {
      "minImportsFromOtherContext": 3,
      "requireMoreThanOwnContext": true
    }]
  },
  "overrides": []
}"#,
    );
}

fn write_architecture_mode_config(
    path: &Path,
    project_root: &str,
    architecture_json: &str,
    rules_json: &str,
    overrides_json: &str,
) {
    write_file(
        path,
        &format!(
            r#"{{
  "version": 1,
  "project": {{
    "root": "{project_root}",
    "include": ["**/*.ts"],
    "exclude": []
  }},
  "architecture": {architecture_json},
  "rules": {rules_json},
  "overrides": {overrides_json}
}}"#
        ),
    );
}

fn write_shotgun_policy_config(path: &Path) {
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
    "codesmells/shotgun-surgery": ["warn", {
      "minCommitCount": 2,
      "minRelatedFiles": 2,
      "minPairCommitCount": 2
    }]
  },
  "overrides": []
}"#,
    );
}

fn write_test_placement_config(path: &Path, rule_json: &str, overrides_json: &str) {
    write_file(
        path,
        &format!(
            r#"{{
  "version": 1,
  "project": {{
    "root": ".",
    "include": ["src/**/*.ts", "tests/**/*.ts", "app/**/*.ts", "it/**/*.ts", "journeys/**/*.ts"],
    "exclude": []
  }},
  "rules": {{
    "repo/test-placement": {rule_json}
  }},
  "overrides": {overrides_json}
}}"#
        ),
    );
}

fn write_path_naming_config(path: &Path, rule_json: &str) {
    write_file(
        path,
        &format!(
            r#"{{
  "version": 1,
  "project": {{
    "root": ".",
    "include": ["src/**/*.ts", "packages/**/*.ts"],
    "exclude": []
  }},
  "rules": {{
    "repo/path-naming": {rule_json}
  }},
  "overrides": []
}}"#
        ),
    );
}

fn write_feature_system_layout_config(path: &Path, rule_json: &str) {
    write_file(
        path,
        &format!(
            r#"{{
  "version": 1,
  "project": {{
    "root": ".",
    "include": ["packages/frontend/src/**/*", "apps/web/src/**/*"],
    "exclude": []
  }},
  "rules": {{
    "frontend/feature-system-layout": {rule_json}
  }},
  "overrides": []
}}"#
        ),
    );
}

fn write_feature_system_public_api_config(path: &Path, rule_json: &str) {
    write_file(
        path,
        &format!(
            r#"{{
  "version": 1,
  "project": {{
    "root": ".",
    "include": ["packages/frontend/src/**/*", "apps/web/src/**/*"],
    "exclude": []
  }},
  "rules": {{
    "frontend/feature-system-public-api": {rule_json}
  }},
  "overrides": []
}}"#
        ),
    );
}

fn write_feature_system_dependency_flow_config(path: &Path, rule_json: &str) {
    write_file(
        path,
        &format!(
            r#"{{
  "version": 1,
  "project": {{
    "root": ".",
    "include": ["packages/frontend/src/**/*", "apps/web/src/**/*"],
    "exclude": []
  }},
  "rules": {{
    "frontend/feature-system-dependency-flow": {rule_json}
  }},
  "overrides": []
}}"#
        ),
    );
}

fn write_feature_system_adapter_contract_config(path: &Path, rule_json: &str) {
    write_file(
        path,
        &format!(
            r#"{{
  "version": 1,
  "project": {{
    "root": ".",
    "include": ["packages/frontend/src/**/*", "apps/web/src/**/*"],
    "exclude": []
  }},
  "rules": {{
    "frontend/feature-system-adapter-contract": {rule_json}
  }},
  "overrides": []
}}"#
        ),
    );
}

fn write_feature_system_query_contract_config(path: &Path, rule_json: &str) {
    write_file(
        path,
        &format!(
            r#"{{
  "version": 1,
  "project": {{
    "root": ".",
    "include": ["packages/frontend/src/**/*", "apps/web/src/**/*"],
    "exclude": []
  }},
  "rules": {{
    "frontend/feature-system-query-contract": {rule_json}
  }},
  "overrides": []
}}"#
        ),
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
    "cleanarch/no-layer-leak": "error",
    "cleanarch/no-cross-context-internal-import": "error",
    "cleanarch/no-forbidden-imports": ["error", {
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
    "cleanarch/unclassified-file": "warn"
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
fn cli_prints_version() {
    onioncry().arg("--version").assert().success().stdout(
        predicate::str::contains(format!("onioncry {}\n", onioncry::CLI_VERSION))
            .and(predicate::str::contains("buildTimestamp").not()),
    );
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
    assert!(config.contains(r#""architecture""#));
    assert!(config.contains(r#""mode": "cleanArchitecture""#));
    assert!(config.contains(r#""cleanArchitecture""#));
    assert!(config.contains(r#""verticalSlice""#));
    assert!(config.contains(r#""contextRoot": "contexts""#));
    assert!(config.contains(r#""sliceRoot": "features""#));
    assert!(config.contains(r#""layerPathAliases""#));
    assert!(config.contains(r#""artifactFolders""#));
    assert!(config.contains(r#""artifactSuffixes""#));
    assert!(config.contains(r#""groupedArtifactFolders""#));
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
    assert!(config.contains(r#""cleanarch/no-layer-leak": "error""#));
    assert!(config.contains(r#""cleanarch/no-cross-context-internal-import": "error""#));
    assert!(config.contains(r#""fromLayer": "domain""#));
    assert!(config.contains(r#""severity": "error""#));
    assert!(config.contains(r#""fromLayer": "application""#));
    assert!(config.contains(r#""severity": "warn""#));
    assert!(config.contains(r#""fromLayer": "infra""#));
    assert!(config.contains(r#""severity": "off""#));
    assert!(!config.contains("codesmells/unresolved-import"));
    assert!(!config.contains("codesmells/circular-dependency"));
    assert!(config.contains(r#""cleanarch/no-framework-in-core": "warn""#));
    assert!(config.contains(r#""cleanarch/no-outer-data-format-in-core": "warn""#));
    assert!(config.contains(r#""cleanarch/no-public-surface-internal-reexport": "warn""#));
    assert!(config.contains(r#""cleanarch/no-context-cycle": "warn""#));
    assert!(config.contains(r#""cleanarch/no-unowned-schema-import": "warn""#));
    assert!(config.contains(r#""cleanarch/artifact-placement": "warn""#));
    assert!(config.contains(r#""solid/no-concrete-dependency": "warn""#));
    assert!(config.contains(r#""codesmells/feature-envy": "warn""#));
    assert!(config.contains(r#""codesmells/shotgun-surgery": "off""#));
    assert!(config.contains(r#""cleanarch/unclassified-file": "warn""#));
    assert!(config.contains(r#""verticalslice/no-cross-slice-internal-import": "warn""#));
    assert!(config.contains(r#""verticalslice/no-global-slice-artifacts": "warn""#));

    let stripped = strip_full_line_jsonc_comments(&config);
    let parsed: Value =
        serde_json::from_str(&stripped).expect("template should parse after comments are stripped");
    assert_eq!(parsed["version"], 1);
    assert!(parsed["project"].is_object());
    assert_eq!(parsed["architecture"]["mode"], "cleanArchitecture");
    assert_eq!(
        parsed["architecture"]["cleanArchitecture"]["contextRoot"],
        "contexts"
    );
    assert_eq!(
        parsed["architecture"]["cleanArchitecture"]["groupedArtifactFolders"][0],
        "use-cases"
    );
    assert!(
        parsed["architecture"]["cleanArchitecture"]["groupedArtifactFolders"]
            .as_array()
            .expect("grouped artifact folders should be an array")
            .iter()
            .any(|folder| folder == "repositories")
    );
    assert!(
        parsed["architecture"]["cleanArchitecture"]["groupedArtifactFolders"]
            .as_array()
            .expect("grouped artifact folders should be an array")
            .iter()
            .any(|folder| folder == "bootstrap")
    );
    assert_eq!(
        parsed["architecture"]["verticalSlice"]["sliceRoot"],
        "features"
    );
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
    assert!(forced.contains(r#""cleanarch/no-layer-leak": "error""#));
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
fn check_discovers_json_config_when_default_jsonc_config_is_missing() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_minimal_config(
        &workspace.path().join(".onioncryrc.json"),
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
}

#[test]
fn check_prefers_jsonc_config_over_json_config() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_minimal_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        &["src/domain/**/*.ts"],
        &[],
    );
    write_minimal_config(
        &workspace.path().join(".onioncryrc.json"),
        ".",
        &["src/application/**/*.ts"],
        &[],
    );
    write_file(
        &workspace.path().join("src/domain/order.ts"),
        "export const id = 1;\n",
    );
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        "export const run = () => undefined;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["fileCount"], 1);
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
        .stderr(
            predicate::str::contains(".onioncryrc.jsonc")
                .and(predicate::str::contains(".onioncryrc.json")),
        );
}

#[test]
fn check_accepts_architecture_modes_and_defaults_missing_mode_to_clean_architecture() {
    let clean_workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &clean_workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{
    "cleanArchitecture": {
      "contextRoot": "contexts",
      "layerPathAliases": {
        "infra": ["infrastructure"]
      }
    },
    "verticalSlice": {
      "sliceRoot": "features"
    }
  }"#,
        r#"{}"#,
        r#"[]"#,
    );
    write_file(
        &clean_workspace.path().join("src/application/use-case.ts"),
        "export const run = () => undefined;\n",
    );

    let clean_result = run_json_check(&clean_workspace, &["check", "--format", "json"]);
    assert_eq!(clean_result["status"], "pass");

    let vertical_workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &vertical_workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{
    "mode": "verticalSlice",
    "verticalSlice": {
      "sliceRoot": "features",
      "publicSurface": ["index.ts", "contracts"],
      "artifactFolders": ["handlers", "adapters", "domain", "__tests__"],
      "artifactSuffixes": {
        "handler": [".handler.ts"],
        "service": [".service.ts"]
      },
      "allowedGlobalFolders": ["app", "config", "lib", "shared", "infra"]
    }
  }"#,
        r#"{}"#,
        r#"[]"#,
    );
    write_file(
        &vertical_workspace
            .path()
            .join("src/features/orders/index.ts"),
        "export const orders = 1;\n",
    );

    let vertical_result = run_json_check(&vertical_workspace, &["check", "--format", "json"]);
    assert_eq!(vertical_result["status"], "pass");
}

#[test]
fn check_rejects_invalid_architecture_mode() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{ "mode": "verticalSlices" }"#,
        r#"{}"#,
        r#"[]"#,
    );
    write_file(
        &workspace.path().join("src/features/orders/index.ts"),
        "export const orders = 1;\n",
    );

    onioncry()
        .current_dir(workspace.path())
        .args(["check"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("verticalSlices")
                .and(predicate::str::contains("cleanArchitecture"))
                .and(predicate::str::contains("verticalSlice")),
        );
}

#[test]
fn check_rejects_architecture_rule_mode_mismatch_before_scanning_files() {
    let vertical_workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &vertical_workspace.path().join(".onioncryrc.jsonc"),
        "missing-src",
        r#"{ "mode": "verticalSlice" }"#,
        r#"{
    "cleanarch/no-layer-leak": "error"
  }"#,
        r#"[]"#,
    );

    onioncry()
        .current_dir(vertical_workspace.path())
        .args(["check"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("cleanarch/no-layer-leak")
                .and(predicate::str::contains("verticalSlice"))
                .and(predicate::str::contains("verticalslice/*")),
        );

    let clean_workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &clean_workspace.path().join(".onioncryrc.jsonc"),
        "missing-src",
        r#"{ "mode": "cleanArchitecture" }"#,
        r#"{}"#,
        r#"[
    {
      "files": ["**/*.ts"],
      "rules": {
        "verticalslice/no-cross-slice-internal-import": "warn"
      }
    }
  ]"#,
    );

    onioncry()
        .current_dir(clean_workspace.path())
        .args(["check"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("verticalslice/no-cross-slice-internal-import")
                .and(predicate::str::contains("cleanArchitecture"))
                .and(predicate::str::contains("cleanarch/*")),
        );
}

#[test]
fn check_runs_architecture_neutral_rules_in_vertical_slice_mode() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{ "mode": "verticalSlice" }"#,
        r#"{
    "repo/path-naming": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace.path().join("src/features/orders/OrderRepo.ts"),
        "export const value = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "fail");
    assert_eq!(result["violations"][0]["rule"], "repo/path-naming");
}

#[test]
fn check_rejects_llm_with_other_output_modes() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    onioncry()
        .current_dir(workspace.path())
        .args(["check", "--llm-mode", "--format", "json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));

    onioncry()
        .current_dir(workspace.path())
        .args(["check", "--llm-mode", "--tip"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn check_rejects_legacy_llm_flag() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    onioncry()
        .current_dir(workspace.path())
        .args(["check", "--llm"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '--llm'"));
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
fn check_pretty_output_uses_clickable_locations_and_explanations() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_layer_config(&workspace.path().join(".onioncryrc.jsonc"));
    let source_file = workspace.path().join("src/application/use-case.ts");
    let target_file = workspace.path().join("src/infra/repo.ts");
    write_file(
        &source_file,
        r#"import { repo } from "../infra/repo";
export const run = repo;
"#,
    );
    write_file(&target_file, "export const repo = 1;\n");

    let output = onioncry()
        .current_dir(workspace.path())
        .args(["check"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let pretty = String::from_utf8(output).expect("pretty output should be utf-8");

    assert!(pretty.contains(&source_file.display().to_string()));
    assert!(pretty.contains("  1:"));
    assert!(pretty.contains(
        "error   application may not import infra through ../infra/repo  cleanarch/no-layer-leak"
    ));
    assert!(!pretty.contains("why:"));
    assert!(!pretty.contains("import: ../infra/repo"));
    assert!(!pretty.contains("target: "));
    assert!(!pretty.contains("help:"));
    assert!(!pretty.contains("tip:"));
    assert!(pretty.contains("1 problem (1 error, 0 warnings)"));
    assert!(pretty.contains("2 files checked"));
    assert!(pretty.trim_end().ends_with("status: fail"));

    let output_with_tips = onioncry()
        .current_dir(workspace.path())
        .args(["check", "--tip"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let pretty_with_tips =
        String::from_utf8(output_with_tips).expect("pretty output should be utf-8");

    assert!(pretty_with_tips.contains(
        "why: Layer rules only allow imports declared in the importing layer's mayImport policy."
    ));
    assert!(pretty_with_tips.contains("import: ../infra/repo"));
    assert!(pretty_with_tips.contains("target: "));
    assert!(pretty_with_tips.contains("src/infra/repo.ts"));
    assert!(pretty_with_tips.contains("tip: add \"infra\" to layers.application.mayImport"));
}

#[test]
fn check_llm_output_groups_repeated_diagnostics_by_fingerprint() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_layer_config(&workspace.path().join(".onioncryrc.jsonc"));
    let first_file = workspace.path().join("src/application/first-use-case.ts");
    let second_file = workspace.path().join("src/application/second-use-case.ts");
    let target_file = workspace.path().join("src/infra/repo.ts");
    write_file(
        &first_file,
        r#"import { repo } from "../infra/repo";
export const first = repo;
"#,
    );
    write_file(
        &second_file,
        r#"import { repo } from "../infra/repo";
export const second = repo;
"#,
    );
    write_file(&target_file, "export const repo = 1;\n");

    let output = onioncry()
        .current_dir(workspace.path())
        .args(["check", "--llm-mode"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let llm = String::from_utf8(output).expect("llm output should be utf-8");
    let lines = llm.lines().collect::<Vec<_>>();

    assert_eq!(lines.first(), Some(&"status: fail"));
    assert_eq!(
        lines
            .get(lines.len().saturating_sub(2))
            .expect("llm report should include a separator before metadata"),
        &onioncry::LLM_REPORT_SEPARATOR
    );
    assert_llm_report_metadata_line(
        lines
            .last()
            .expect("llm report should include metadata as the last line"),
    );
    assert!(llm.contains("status: fail"));
    assert!(!llm.contains("buildTimestamp"));
    assert!(llm.contains("filesChecked: 3"));
    assert!(llm.contains("problemCount: 2"));
    assert!(llm.contains("groupCount: 1"));
    assert!(llm.contains("count: 2"));
    assert!(llm.contains("severity: error"));
    assert!(llm.contains("rule: cleanarch/no-layer-leak"));
    assert!(llm.contains("message: application may not import infra through ../infra/repo"));
    assert!(llm.contains(
        "why: Layer rules only allow imports declared in the importing layer's mayImport policy."
    ));
    assert!(llm.contains("import: ../infra/repo"));
    assert!(llm.contains("layers: application -> infra"));
    assert!(llm.contains("target: "));
    assert!(llm.contains("src/infra/repo.ts"));
    assert!(llm.contains("tip: add \"infra\" to layers.application.mayImport"));
    assert!(llm.contains("src/application/first-use-case.ts:1:"));
    assert!(llm.contains("src/application/second-use-case.ts:1:"));
}

#[test]
fn check_delegates_unresolved_local_import_diagnostics_to_general_linters() {
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

    let check = run_json_check(&workspace, &["check", "--format", "json"]);
    assert_eq!(check["status"], "pass");
    assert_eq!(check["summary"]["warningCount"], 0);
    assert_eq!(check["violations"].as_array().unwrap().len(), 0);

    let explain = run_json_explain(
        &workspace,
        &["explain", "src/application/use-case.ts", "--format", "json"],
    );
    let imports = explain["imports"]
        .as_array()
        .expect("imports should be an array");
    let unresolved_count = imports
        .iter()
        .filter(|import| import["resolution"] == "unresolvedLocal")
        .count();
    assert_eq!(unresolved_count, 5);
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
import type { Gateway } from "./erp-gateway.port";
import type { Env } from "@app/application/env.schema";
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
        &workspace.path().join("src/application/erp-gateway.port.ts"),
        "export type Gateway = {};\n",
    );
    write_file(
        &workspace.path().join("src/application/env.schema.ts"),
        "export type Env = {};\n",
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
        violation["rule"] == "cleanarch/no-layer-leak"
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
    "cleanarch/unclassified-file": "warn"
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
        violation["rule"] == "cleanarch/ambiguous-layer"
            && violation["severity"] == "error"
            && violation["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("src/ambiguous/value.ts"))
            && violation["matchedLayers"]
                .as_array()
                .is_some_and(|layers| layers.len() == 2)
    }));
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "cleanarch/unclassified-file"
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
    assert_eq!(result["violations"][0]["rule"], "cleanarch/no-layer-leak");
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
fn check_accepts_default_test_placement_layout() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_test_placement_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
        "[]",
    );
    write_file(
        &workspace.path().join("src/orders/__tests__/order.test.ts"),
        "export const orderTest = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("tests/integration/orders/repository.spec.ts"),
        "export const repositorySpec = 1;\n",
    );
    write_file(
        &workspace.path().join("tests/e2e/orders/checkout.test.ts"),
        "export const checkoutTest = 1;\n",
    );
    write_file(
        &workspace.path().join("src/orders/order.ts"),
        "export const order = 1;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["fileCount"], 4);
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_reports_misplaced_test_files_with_suggestions() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_test_placement_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
        "[]",
    );
    write_file(
        &workspace.path().join("src/orders/order.test.ts"),
        "export const orderTest = 1;\n",
    );
    write_file(
        &workspace.path().join("tests/integration/order.spec.ts"),
        "export const integrationSpec = 1;\n",
    );
    write_file(
        &workspace.path().join("tests/unit/orders/order.test.ts"),
        "export const unitTest = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/orders/__tests__/allowed.spec.ts"),
        "export const allowedSpec = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["fileCount"], 4);
    assert_eq!(result["summary"]["errorCount"], 3);
    assert_eq!(violations.len(), 3);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "repo/test-placement"
            && violation["severity"] == "error"
            && violation["message"] == "test file is not in an allowed test location"
    }));
    assert!(violations.iter().any(|violation| {
        violation["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("src/orders/order.test.ts"))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("__tests__"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("tests/integration/order.spec.ts"))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("tests/integration/<context>/"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("tests/unit/orders/order.test.ts"))
            && violation["suggestion"].as_str().is_some_and(|suggestion| {
                suggestion.contains("tests/integration")
                    && suggestion.contains("tests/e2e")
                    && suggestion.contains("__tests__")
            })
    }));
}

#[test]
fn check_applies_test_placement_overrides_without_changing_file_universe() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_test_placement_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
        r#"[
    {
      "files": ["src/orders/legacy.test.ts"],
      "rules": {
        "repo/test-placement": "off"
      }
    },
    {
      "files": ["src/orders/soft.spec.ts"],
      "rules": {
        "repo/test-placement": "warn"
      }
    }
  ]"#,
    );
    write_file(
        &workspace.path().join("src/orders/legacy.test.ts"),
        "export const legacy = 1;\n",
    );
    write_file(
        &workspace.path().join("src/orders/soft.spec.ts"),
        "export const soft = 1;\n",
    );
    write_file(
        &workspace.path().join("src/orders/strict.test.ts"),
        "export const strict = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["summary"]["fileCount"], 3);
    assert_eq!(result["summary"]["errorCount"], 1);
    assert_eq!(result["summary"]["warningCount"], 1);
    assert_eq!(violations.len(), 2);
    assert!(!violations.iter().any(|violation| {
        violation["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("src/orders/legacy.test.ts"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["severity"] == "warn"
            && violation["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("src/orders/soft.spec.ts"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["severity"] == "error"
            && violation["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("src/orders/strict.test.ts"))
    }));
}

#[test]
fn check_accepts_test_placement_severity_and_options_forms() {
    let off_workspace = TempDir::new().expect("workspace should be creatable");
    write_test_placement_config(
        &off_workspace.path().join(".onioncryrc.jsonc"),
        r#""off""#,
        "[]",
    );
    write_file(
        &off_workspace.path().join("src/orders/order.test.ts"),
        "export const orderTest = 1;\n",
    );
    let off_result = run_json_check(&off_workspace, &["check", "--format", "json"]);
    assert_eq!(off_result["summary"]["violationCount"], 0);

    let warn_workspace = TempDir::new().expect("workspace should be creatable");
    write_test_placement_config(
        &warn_workspace.path().join(".onioncryrc.jsonc"),
        r#""warn""#,
        "[]",
    );
    write_file(
        &warn_workspace.path().join("src/orders/order.test.ts"),
        "export const orderTest = 1;\n",
    );
    let warn_result = run_json_check(&warn_workspace, &["check", "--format", "json"]);
    assert_eq!(warn_result["status"], "pass");
    assert_eq!(warn_result["summary"]["warningCount"], 1);

    let options_workspace = TempDir::new().expect("workspace should be creatable");
    write_test_placement_config(
        &options_workspace.path().join(".onioncryrc.jsonc"),
        r#"["error", {
      "sourceRoots": ["app"],
      "unitTestDirectories": ["specs"],
      "integrationRoots": ["it"],
      "e2eRoots": ["journeys"],
      "testFileSuffixes": [".check.ts"]
    }]"#,
        "[]",
    );
    write_file(
        &options_workspace
            .path()
            .join("app/orders/specs/order.check.ts"),
        "export const orderCheck = 1;\n",
    );
    write_file(
        &options_workspace
            .path()
            .join("it/orders/repository.check.ts"),
        "export const repositoryCheck = 1;\n",
    );
    write_file(
        &options_workspace
            .path()
            .join("journeys/orders/checkout.check.ts"),
        "export const checkoutCheck = 1;\n",
    );

    let options_result = run_json_check(&options_workspace, &["check", "--format", "json"]);

    assert_eq!(options_result["status"], "pass");
    assert_eq!(options_result["summary"]["fileCount"], 3);
    assert_eq!(options_result["summary"]["violationCount"], 0);
}

#[test]
fn check_accepts_default_path_naming_layout_without_inspecting_symbols() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_path_naming_config(&workspace.path().join(".onioncryrc.jsonc"), r#""error""#);
    write_file(
        &workspace
            .path()
            .join("src/billing/domain/entities/order.ts"),
        r#"export class BAD_symbol_name {}
export const MIXED_SYMBOL_NAME = 1;
"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/billing/application/use-cases/create-order.use-case.ts"),
        "export const run = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/billing/infra/repositories/order.repository.ts"),
        "export const repository = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/billing/application/dtos/order.dto.ts"),
        "export type OrderDto = { id: string };\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["fileCount"], 4);
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_reports_invalid_path_naming_conventions() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_path_naming_config(&workspace.path().join(".onioncryrc.jsonc"), r#""error""#);
    write_file(
        &workspace
            .path()
            .join("src/orders/infrastructure/repository/OrderRepo.ts"),
        "export const value = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/billing/infra/repositories/order.ts"),
        "export const repository = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/CustomerProfile/domain/entities/order.ts"),
        "export const order = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(
        violations
            .iter()
            .all(|violation| violation["rule"] == "repo/path-naming")
    );
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("feature directory \"orders\""))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("singular"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("infrastructure"))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("infra"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("collection directory \"repository\""))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("repositories"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("file name \"OrderRepo.ts\""))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("files in \"repositories\""))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains(".repository"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("directory segment \"CustomerProfile\""))
    }));
}

#[test]
fn check_accepts_custom_path_naming_options() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_path_naming_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#"["error", {
      "featureRoots": ["packages/app/src/features"],
      "layerDirectories": ["core", "usecases", "infrastructure", "shared"],
      "collectionDirectories": ["models", "repos"],
      "suffixes": {
        "repos": ".repo"
      }
    }]"#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/app/src/features/account/core/models/order.ts"),
        "export const order = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/app/src/features/account/infrastructure/repos/order.repo.ts"),
        "export const repo = 1;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["fileCount"], 2);
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_accepts_collection_suffix_before_test_or_spec_suffix() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_path_naming_config(&workspace.path().join(".onioncryrc.jsonc"), r#""error""#);
    write_file(
        &workspace
            .path()
            .join("src/billing/application/services/tax-import.service.test.ts"),
        "export const taxImportService = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/billing/application/use-cases/submit-order.use-case.spec.ts"),
        "export const submitOrderUseCase = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/billing/domain/value-objects/timestamp.value-object.test.ts"),
        "export const timestamp = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/billing/domain/events/order-created.event.spec.ts"),
        "export const orderCreatedEvent = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/billing/infra/repositories/__tests__/audit-event.repository.test.ts"),
        "export const auditEventRepository = 1;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["fileCount"], 5);
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_accepts_context_first_clean_architecture_artifact_placement() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "contexts",
      "layerPathAliases": {
        "infra": ["infra", "infrastructure"]
      }
    }
  }"#,
        r#"{
    "cleanarch/artifact-placement": ["error", { "note": "migration gate" }]
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/contexts/billing/application/use-cases/create-invoice.use-case.ts"),
        "export const createInvoice = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/contexts/billing/domain/services/tax.service.ts"),
        "export const tax = 1;\n",
    );
    write_file(
        &workspace.path().join("src/domain/entities/money.entity.ts"),
        "export const money = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/infrastructure/repositories/audit.repository.ts"),
        "export const audit = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/infra/repositories/drizzle/customer.repository.ts"),
        "export const customerRepository = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/use-cases/catalog/sync-products.use-case.ts"),
        "export const syncProducts = () => undefined;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_accepts_clean_architecture_artifacts_with_source_prefix() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        r#"{
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "src/contexts"
    }
  }"#,
        r#"{
    "cleanarch/artifact-placement": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/contexts/billing/application/use-cases/create-invoice.use-case.ts"),
        "export const createInvoice = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/use-cases/catalog/sync-products.use-case.ts"),
        "export const syncProducts = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/infra/repositories/drizzle/customer.repository.ts"),
        "export const customerRepository = 1;\n",
    );
    write_file(
        &workspace.path().join("src/domain/entities/money.entity.ts"),
        "export const money = 1;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_reports_flat_contextless_clean_architecture_use_case_lists() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        r#"{
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "src/contexts"
    }
  }"#,
        r#"{
    "cleanarch/artifact-placement": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/application/use-cases/create-invoice.use-case.ts"),
        "export const createInvoice = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/use-cases/submit-invoice.use-case.ts"),
        "export const submitInvoice = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/use-cases/catalog/sync-products.use-case.ts"),
        "export const syncProducts = () => undefined;\n",
    );
    write_file(
        &workspace.path().join("src/application/use-cases/index.ts"),
        "export * from \"./catalog/sync-products.use-case\";\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["errorCount"], 2);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "cleanarch/artifact-placement"
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("src/application/use-cases/<group>"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["file"].as_str().is_some_and(|file| {
            file.ends_with("src/application/use-cases/create-invoice.use-case.ts")
        })
    }));
}

#[test]
fn check_accepts_single_direct_contextless_grouped_artifact_file() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        r#"{
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "src/contexts",
      "artifactFolders": {
        "domain": ["entities", "value-objects", "ports"],
        "application": ["use-cases", "ports"]
      },
      "groupedArtifactFolders": ["use-cases", "entities", "value-objects", "ports"]
    }
  }"#,
        r#"{
    "cleanarch/artifact-placement": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace.path().join("src/domain/entities/product.ts"),
        "export const product = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/use-cases/import-products.ts"),
        "export const importProducts = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/ports/catalog-port.ts"),
        "export const catalogPort = {};\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_reports_contextless_clean_architecture_capability_folders_under_domain() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        r#"{
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "src/contexts",
      "artifactFolders": {
        "domain": ["entities", "value-objects", "ports"]
      },
      "groupedArtifactFolders": ["entities", "value-objects", "ports"]
    }
  }"#,
        r#"{
    "cleanarch/artifact-placement": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/domain/classification-exports/classification-export.ts"),
        "export const classificationExport = 1;\n",
    );
    write_file(
        &workspace.path().join("src/domain/reviews/tax-review.ts"),
        "export const taxReview = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/domain/reviews/tax-review-warnings.ts"),
        "export const taxReviewWarnings = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/domain/entities/direct-entity.ts"),
        "export const directEntity = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["errorCount"], 3);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "cleanarch/artifact-placement" && violation["toLayer"] == "domain"
    }));
    assert!(violations.iter().any(|violation| {
        violation["file"].as_str().is_some_and(|file| {
            file.ends_with("src/domain/classification-exports/classification-export.ts")
        }) && violation["suggestion"].as_str().is_some_and(|suggestion| {
            suggestion
                .contains("src/domain/entities or src/domain/value-objects or src/domain/ports")
                && !suggestion.contains("classification-exports")
        })
    }));
    assert!(violations.iter().any(|violation| {
        violation["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("src/domain/reviews/tax-review.ts"))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("src/domain/entities/reviews"))
    }));
}

#[test]
fn check_reports_contextless_clean_architecture_capability_folders_under_application() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        r#"{
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "src/contexts",
      "artifactFolders": {
        "application": ["use-cases", "ports"]
      }
    }
  }"#,
        r#"{
    "cleanarch/artifact-placement": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/application/reviews/get-tax-review.ts"),
        "export const getTaxReview = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/import-sessions/import-session-ports.ts"),
        "export const importSessionPorts = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/use-cases/reviews/list-tax-reviews.ts"),
        "export const listTaxReviews = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/ports/reviews/tax-review-ports.ts"),
        "export const taxReviewPorts = {};\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["errorCount"], 2);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "cleanarch/artifact-placement" && violation["toLayer"] == "application"
    }));
    assert!(violations.iter().any(|violation| {
        violation["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("src/application/reviews/get-tax-review.ts"))
            && violation["suggestion"].as_str().is_some_and(|suggestion| {
                suggestion.contains("src/application/use-cases or src/application/ports")
            })
    }));
    assert!(violations.iter().any(|violation| {
        violation["file"].as_str().is_some_and(|file| {
            file.ends_with("src/application/import-sessions/import-session-ports.ts")
        }) && violation["suggestion"]
            .as_str()
            .is_some_and(|suggestion| suggestion.contains("src/application/ports"))
    }));
}

#[test]
fn check_reports_contextless_clean_architecture_capability_folders_under_infra() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        r#"{
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "src/contexts",
      "artifactFolders": {
        "infra": ["repositories", "adapters", "bootstrap"]
      },
      "groupedArtifactFolders": ["repositories", "adapters", "bootstrap"]
    }
  }"#,
        r#"{
    "cleanarch/artifact-placement": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/infra/reviews/drizzle-tax-review-repository.ts"),
        "export const drizzleTaxReviewRepository = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/infra/reviews/fake-tax-provider-gateway.ts"),
        "export const fakeTaxProviderGateway = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/infra/tax-configurations/drizzle-tax-configuration-catalog.ts"),
        "export const drizzleTaxConfigurationCatalog = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/infra/repositories/audit-repository.ts"),
        "export const auditRepository = {};\n",
    );
    write_file(
        &workspace.path().join("src/infra/adapters/http-client.ts"),
        "export const httpClient = {};\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["errorCount"], 3);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "cleanarch/artifact-placement" && violation["toLayer"] == "infra"
    }));
    assert!(violations.iter().any(|violation| {
        violation["file"].as_str().is_some_and(|file| {
            file.ends_with("src/infra/tax-configurations/drizzle-tax-configuration-catalog.ts")
        }) && violation["suggestion"].as_str().is_some_and(|suggestion| {
            suggestion.contains("src/infra/repositories")
                && !suggestion.contains("tax-configurations")
        })
    }));
    assert!(violations.iter().any(|violation| {
        violation["file"].as_str().is_some_and(|file| {
            file.ends_with("src/infra/reviews/drizzle-tax-review-repository.ts")
        }) && violation["suggestion"]
            .as_str()
            .is_some_and(|suggestion| suggestion.contains("src/infra/repositories/reviews"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("src/infra/reviews/fake-tax-provider-gateway.ts"))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("src/infra/adapters/reviews"))
    }));
}

#[test]
fn check_reports_source_prefixed_layer_first_clean_architecture_artifacts() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        ".",
        r#"{
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "src/contexts"
    }
  }"#,
        r#"{
    "cleanarch/artifact-placement": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/application/billing/create-invoice.use-case.ts"),
        "export const createInvoice = () => undefined;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["errorCount"], 1);
    assert_eq!(violations[0]["rule"], "cleanarch/artifact-placement");
    assert!(
        violations[0]["suggestion"]
            .as_str()
            .is_some_and(|suggestion| suggestion.contains("src/application/use-cases"))
    );
}

#[test]
fn check_reports_clean_architecture_artifact_placement_violations() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{ "mode": "cleanArchitecture" }"#,
        r#"{
    "cleanarch/artifact-placement": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/application/billing/create-invoice.use-case.ts"),
        "export const createInvoice = () => undefined;\n",
    );
    write_file(
        &workspace.path().join("src/sales/entities/order.entity.ts"),
        "export const order = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/contexts/catalog/repositories/catalog.repository.ts"),
        "export const catalog = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["errorCount"], 3);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "cleanarch/artifact-placement"
            && violation["severity"] == "error"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("cleanArchitecture"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("use-cases"))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("application/use-cases"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("entities"))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("contexts/sales/domain"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("repositories"))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("contexts/catalog/infra"))
    }));
}

#[test]
fn check_applies_clean_architecture_artifact_options_and_overrides() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "modules",
      "layerPathAliases": {
        "infra": ["infrastructure"]
      },
      "artifactFolders": {
        "infra": ["repos"]
      },
      "artifactSuffixes": {
        "repository": [".repo.ts"],
        "useCase": [".use-case.ts"]
      }
    }
  }"#,
        r#"{
    "cleanarch/artifact-placement": "error"
  }"#,
        r#"[
    {
      "files": ["application/legacy/**"],
      "rules": {
        "cleanarch/artifact-placement": "off"
      }
    }
  ]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/modules/billing/infrastructure/repos/invoice.repo.ts"),
        "export const invoiceRepo = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/legacy/create-invoice.use-case.ts"),
        "export const legacy = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/billing/create-invoice.use-case.ts"),
        "export const createInvoice = () => undefined;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["summary"]["errorCount"], 1);
    assert_eq!(violations[0]["rule"], "cleanarch/artifact-placement");
    assert!(
        violations[0]["suggestion"]
            .as_str()
            .is_some_and(|suggestion| suggestion.contains("application/use-cases"))
    );
}

#[test]
fn check_reports_cross_slice_internal_imports_but_allows_public_surface_imports() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{ "mode": "verticalSlice" }"#,
        r#"{
    "verticalslice/no-cross-slice-internal-import": ["error", { "note": "slice boundary" }]
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace.path().join("src/features/billing/index.ts"),
        "export const billing = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/features/billing/contracts/billing-event.ts"),
        "export type BillingEvent = { id: string };\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/features/billing/handlers/create-billing.handler.ts"),
        "export type BillingHandler = () => void;\nexport const createBilling = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/features/orders/domain/order.service.ts"),
        "export const orderService = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/features/orders/handlers/create-order.handler.ts"),
        r#"import { billing } from "../../billing";
import type { BillingEvent } from "../../billing/contracts/billing-event";
import { orderService } from "../domain/order.service";
import type { BillingHandler } from "../../billing/handlers/create-billing.handler";
export { createBilling } from "../../billing/handlers/create-billing.handler";
export const createOrder = { billing, orderService } satisfies { billing: unknown; orderService: unknown };
export type OrderBillingHandler = BillingHandler;
export type OrderBillingEvent = BillingEvent;
"#,
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["errorCount"], 2);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "verticalslice/no-cross-slice-internal-import"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("verticalSlice"))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("features/billing"))
    }));
    assert!(violations.iter().all(|violation| {
        violation["importSpecifier"] == "../../billing/handlers/create-billing.handler"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("orders"))
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("billing"))
    }));
}

#[test]
fn check_accepts_custom_vertical_slice_public_surface_and_root_level_slices() {
    let custom_workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &custom_workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{
    "mode": "verticalSlice",
    "verticalSlice": {
      "sliceRoot": "modules",
      "publicSurface": ["public.ts", "api"],
      "artifactFolders": ["handlers", "domain"],
      "artifactSuffixes": {
        "handler": [".handler.ts"],
        "service": [".service.ts"]
      }
    }
  }"#,
        r#"{
    "verticalslice/no-cross-slice-internal-import": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &custom_workspace
            .path()
            .join("src/modules/billing/public.ts"),
        "export const billing = 1;\n",
    );
    write_file(
        &custom_workspace
            .path()
            .join("src/modules/billing/api/billing-contract.ts"),
        "export type BillingContract = { id: string };\n",
    );
    write_file(
        &custom_workspace
            .path()
            .join("src/modules/orders/handlers/create-order.handler.ts"),
        r#"import { billing } from "../../billing/public";
import type { BillingContract } from "../../billing/api/billing-contract";
export const createOrder = billing;
export type OrderContract = BillingContract;
"#,
    );

    let custom_result = run_json_check(&custom_workspace, &["check", "--format", "json"]);
    assert_eq!(custom_result["status"], "pass");
    assert_eq!(custom_result["summary"]["violationCount"], 0);

    let root_workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &root_workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{
    "mode": "verticalSlice",
    "verticalSlice": {
      "sliceRoot": ".",
      "allowedGlobalFolders": ["app", "config", "shared"]
    }
  }"#,
        r#"{
    "verticalslice/no-global-slice-artifacts": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &root_workspace
            .path()
            .join("src/orders/handlers/create-order.handler.ts"),
        "export const createOrder = () => undefined;\n",
    );
    write_file(
        &root_workspace.path().join("src/app/server.handler.ts"),
        "export const server = 1;\n",
    );

    let root_result = run_json_check(&root_workspace, &["check", "--format", "json"]);
    assert_eq!(root_result["status"], "pass");
    assert_eq!(root_result["summary"]["violationCount"], 0);
}

#[test]
fn check_reports_global_slice_artifacts_outside_slice_root() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{
    "mode": "verticalSlice",
    "verticalSlice": {
      "sliceRoot": "features",
      "allowedGlobalFolders": ["app", "config", "lib", "shared", "infra"]
    }
  }"#,
        r#"{
    "verticalslice/no-global-slice-artifacts": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/features/orders/handlers/create-order.handler.ts"),
        "export const createOrder = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/use-cases/create-order.use-case.ts"),
        "export const createOrderUseCase = () => undefined;\n",
    );
    write_file(
        &workspace.path().join("src/shared/create-order.use-case.ts"),
        "export const shared = 1;\n",
    );
    write_file(
        &workspace.path().join("src/infra/payment.adapter.ts"),
        "export const payment = 1;\n",
    );
    write_file(
        &workspace.path().join("src/domain/order.ts"),
        "export const order = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["errorCount"], 1);
    assert_eq!(
        violations[0]["rule"],
        "verticalslice/no-global-slice-artifacts"
    );
    assert!(violations[0]["message"].as_str().is_some_and(|message| {
        message.contains("verticalSlice")
            && message.contains("use-cases")
            && message.contains("features")
    }));
}

#[test]
fn check_applies_global_slice_artifact_allowed_global_folders() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{
    "mode": "verticalSlice",
    "verticalSlice": {
      "sliceRoot": "features",
      "allowedGlobalFolders": ["application"]
    }
  }"#,
        r#"{
    "verticalslice/no-global-slice-artifacts": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/application/use-cases/create-order.use-case.ts"),
        "export const createOrderUseCase = () => undefined;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_accepts_disabled_vertical_slice_rules() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{ "mode": "verticalSlice" }"#,
        r#"{
    "verticalslice/no-cross-slice-internal-import": "off",
    "verticalslice/no-global-slice-artifacts": "off"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/features/billing/handlers/create-billing.handler.ts"),
        "export const createBilling = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/features/orders/handlers/create-order.handler.ts"),
        "import { createBilling } from '../../billing/handlers/create-billing.handler';\nexport const createOrder = createBilling;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/application/use-cases/create-order.use-case.ts"),
        "export const createOrderUseCase = () => undefined;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_accepts_path_naming_severity_forms() {
    let off_workspace = TempDir::new().expect("workspace should be creatable");
    write_path_naming_config(&off_workspace.path().join(".onioncryrc.jsonc"), r#""off""#);
    write_file(
        &off_workspace.path().join("src/orders/OrderRepo.ts"),
        "export const value = 1;\n",
    );
    let off_result = run_json_check(&off_workspace, &["check", "--format", "json"]);
    assert_eq!(off_result["summary"]["violationCount"], 0);

    let warn_workspace = TempDir::new().expect("workspace should be creatable");
    write_path_naming_config(
        &warn_workspace.path().join(".onioncryrc.jsonc"),
        r#""warn""#,
    );
    write_file(
        &warn_workspace.path().join("src/orders/OrderRepo.ts"),
        "export const value = 1;\n",
    );

    let warn_result = run_json_check(&warn_workspace, &["check", "--format", "json"]);

    assert_eq!(warn_result["status"], "pass");
    assert_eq!(warn_result["summary"]["errorCount"], 0);
    assert!(
        warn_result["summary"]["warningCount"]
            .as_u64()
            .is_some_and(|count| count > 0)
    );
}

#[test]
fn check_accepts_default_feature_system_layout() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_layout_config(&workspace.path().join(".onioncryrc.jsonc"), r#""error""#);
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/index.ts"),
        "export { BillingCard } from './components/billing-card';\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/components/billing-card.tsx"),
        "export function BillingCard() { return null; }\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/lib/query-options.ts"),
        "export const billingQueryOptions = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/hooks/use-billing.ts"),
        "export const useBilling = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-api.ts"),
        "export const billingApi = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/contexts/billing-context.ts"),
        "export const billingContext = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/stores/billing-store.ts"),
        "export const billingStore = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/guards/billing-guard.ts"),
        "export const billingGuard = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/billing.css"),
        ".billing {}\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/components/ui/button.tsx"),
        "export function Button() { return null; }\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_reports_invalid_feature_system_layout() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_layout_config(&workspace.path().join(".onioncryrc.jsonc"), r#""error""#);
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/orders/components/order-card.tsx"),
        "export function OrderCard() { return null; }\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/components/orders/order-card.tsx"),
        "export function SharedOrderCard() { return null; }\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/features/catalog/components/catalog-card.tsx"),
        "export function CatalogCard() { return null; }\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(
        violations
            .iter()
            .all(|violation| violation["rule"] == "frontend/feature-system-layout")
    );
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("missing required lib/ directory"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("missing root index.ts"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"].as_str().is_some_and(|message| {
            message.contains("feature-specific frontend component is outside a feature system")
        }) && violation["file"].as_str().is_some_and(|file| {
            file.ends_with("packages/frontend/src/components/orders/order-card.tsx")
        })
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("legacy feature root"))
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("systems/catalog"))
    }));
}

#[test]
fn check_reports_feature_system_surface_css_violations() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_layout_config(&workspace.path().join(".onioncryrc.jsonc"), r#""error""#);
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/profile/index.ts"),
        "export const profile = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/profile/components/profile-card.tsx"),
        "export function ProfileCard() { return null; }\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/profile/lib/profile-options.ts"),
        "export const profileOptions = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/profile/components/profile.css"),
        ".profile {}\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/profile/profile-surface.css"),
        ".profileSurface {}\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("must live at the system root"))
            && violation["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("components/profile.css"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("should be named \"profile.css\""))
            && violation["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("profile-surface.css"))
    }));
}

#[test]
fn check_accepts_custom_feature_system_layout_options() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_layout_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#"["error", {
      "systemsRoots": ["apps/web/src/modules"],
      "requiredDirectories": ["ui", "logic"],
      "allowedSharedComponentRoots": ["apps/web/src/ui/primitives"],
      "legacyRoots": ["apps/web/src/old-features"],
      "componentDirectories": ["ui"],
      "rootIndexFile": "public.ts",
      "surfaceCssNameTemplate": "{domain}.module.css"
    }]"#,
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/public.ts"),
        "export { AccountCard } from './ui/account-card';\n",
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/ui/account-card.tsx"),
        "export function AccountCard() { return null; }\n",
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/logic/account-options.ts"),
        "export const accountOptions = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/accounts.module.css"),
        ".accounts {}\n",
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/ui/primitives/button.tsx"),
        "export function Button() { return null; }\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_rejects_feature_system_public_api_wildcard_reexports() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_public_api_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/index.ts"),
        "export * from './components/billing-card';\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/components/billing-card.tsx"),
        "export function BillingCard() { return null; }\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "frontend/feature-system-public-api"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("wildcard re-exports"))
            && violation["file"].as_str().is_some_and(|file| {
                file.ends_with("packages/frontend/src/systems/billing/index.ts")
            })
    }));
}

#[test]
fn check_accepts_named_feature_system_public_exports_and_route_imports() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_public_api_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/index.ts"),
        "export { BillingCard } from './components/billing-card';\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/components/billing-card.tsx"),
        "export function BillingCard() { return null; }\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/routes/billing-route.tsx"),
        "import { BillingCard } from '../systems/billing';\nexport const Route = BillingCard;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_reports_cross_system_internal_imports_but_allows_same_system_imports() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_public_api_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/components/billing-card.tsx"),
        "import { formatBilling } from '../lib/format-billing';\nexport function BillingCard() { return formatBilling(); }\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/lib/format-billing.ts"),
        "export function formatBilling() { return null; }\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/orders/components/order-card.tsx"),
        "import { BillingCard } from '../../billing/components/billing-card';\nexport const OrderCard = BillingCard;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(violations.len(), 1);
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "frontend/feature-system-public-api"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("billing system internal file"))
            && violation["targetFile"].as_str().is_some_and(|file| {
                file.ends_with("packages/frontend/src/systems/billing/components/billing-card.tsx")
            })
    }));
}

#[test]
fn check_reports_route_feature_system_internal_imports() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_public_api_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/components/billing-card.tsx"),
        "export function BillingCard() { return null; }\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/routes/billing-route.tsx"),
        "import { BillingCard } from '../systems/billing/components/billing-card';\nexport const Route = BillingCard;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "frontend/feature-system-public-api"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("route may not import"))
    }));
}

#[test]
fn check_accepts_custom_feature_system_public_api_options() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_public_api_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#"["error", {
      "systemsRoots": ["apps/web/src/modules"],
      "routeRoots": ["apps/web/src/pages"],
      "allowedPublicEntryPoints": ["public.ts"]
    }]"#,
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/public.ts"),
        "export { AccountCard } from './components/account-card';\n",
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/components/account-card.tsx"),
        "export function AccountCard() { return null; }\n",
    );
    write_file(
        &workspace.path().join("apps/web/src/pages/accounts.tsx"),
        "import { AccountCard } from '../modules/accounts/public';\nexport const AccountsPage = AccountCard;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_accepts_feature_system_dependency_flow_defaults() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_dependency_flow_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-client.ts"),
        "export const billingClient = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-api.ts"),
        "import { billingClient } from './billing-client';\nimport { formatBilling } from '../lib/format-billing';\nexport const billingApi = { billingClient, formatBilling };\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/lib/format-billing.ts"),
        "export function formatBilling() { return null; }\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/lib/query-options.ts"),
        "import { billingApi } from '../adapters/billing-api';\nexport const billingQueryOptions = { billingApi };\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/hooks/use-billing.ts"),
        "import { billingQueryOptions } from '../lib/query-options';\nexport const useBilling = () => billingQueryOptions;\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/contexts/billing-context.ts"),
        "import { useBilling } from '../hooks/use-billing';\nimport { formatBilling } from '../lib/format-billing';\nexport const billingContext = { useBilling, formatBilling };\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/stores/billing-store.ts"),
        "import { billingApi } from '../adapters/billing-api';\nimport { billingQueryOptions } from '../lib/query-options';\nexport const billingStore = { billingApi, billingQueryOptions };\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/components/billing-button.tsx"),
        "export function BillingButton() { return null; }\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/components/billing-card.tsx"),
        "import { BillingButton } from './billing-button';\nimport { useBilling } from '../hooks/use-billing';\nimport { billingContext } from '../contexts/billing-context';\nimport { billingStore } from '../stores/billing-store';\nimport { formatBilling } from '../lib/format-billing';\nexport const BillingCard = { BillingButton, useBilling, billingContext, billingStore, formatBilling };\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/index.ts"),
        "export { BillingCard } from './components/billing-card';\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/routes/billing-route.tsx"),
        "import { BillingCard } from '../systems/billing';\nexport const Route = BillingCard;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_reports_feature_system_dependency_flow_violations() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_dependency_flow_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-api.ts"),
        "import { useBilling } from '../hooks/use-billing';\nexport const billingApi = { useBilling };\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/hooks/use-billing.ts"),
        "export const useBilling = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/lib/format-billing.ts"),
        "import { billingApi } from '../adapters/billing-api';\nexport const formatBilling = billingApi;\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/components/billing-card.tsx"),
        "import { billingApi } from '../adapters/billing-api';\nexport const BillingCard = billingApi;\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/stores/billing-store.ts"),
        "import { BillingCard } from '../components/billing-card';\nexport const billingStore = BillingCard;\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/routes/billing-route.tsx"),
        "import { BillingCard } from '../systems/billing/components/billing-card';\nexport const Route = BillingCard;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "frontend/feature-system-dependency-flow"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("adapters may not import hooks"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("lib may not import adapters"))
            && violation["file"].as_str().is_some_and(|file| {
                file.ends_with("packages/frontend/src/systems/billing/lib/format-billing.ts")
            })
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("components may not import adapters"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("stores may not import components"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("routes may not import components"))
    }));
}

#[test]
fn check_accepts_custom_feature_system_dependency_flow_options() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_dependency_flow_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#"["error", {
      "systemsRoots": ["apps/web/src/modules"],
      "routeRoots": ["apps/web/src/pages"],
      "allowedPublicEntryPoints": ["public.ts"],
      "allowedImports": {
        "components": ["components", "adapters"]
      }
    }]"#,
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/adapters/accounts-api.ts"),
        "export const accountsApi = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/components/account-card.tsx"),
        "import { accountsApi } from '../adapters/accounts-api';\nexport const AccountCard = accountsApi;\n",
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/public.ts"),
        "export { AccountCard } from './components/account-card';\n",
    );
    write_file(
        &workspace.path().join("apps/web/src/pages/accounts.tsx"),
        "import { AccountCard } from '../modules/accounts/public';\nexport const AccountsPage = AccountCard;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_accepts_default_feature_system_adapter_contract() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_adapter_contract_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-client.ts"),
        "export const billingClient = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-api.ts"),
        r#"export class BillingApiError extends Error {}
export const billingApi = {
  async list(signal?: AbortSignal) {
    return fetch("/api/billing", { signal });
  }
};
"#,
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_reports_missing_feature_system_adapter_file_and_exports() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_adapter_contract_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-client.ts"),
        "export const billingClient = {};\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/orders/adapters/orders-api.ts"),
        "export const ordersAdapter = {};\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "frontend/feature-system-adapter-contract"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("adapters/billing-api.ts"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("ordersApi"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("OrdersApiError"))
    }));
}

#[test]
fn check_reports_feature_system_adapter_cancellation_violations() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_adapter_contract_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-api.ts"),
        r#"export class BillingApiError extends Error {}
export const billingApi = {
  async list() {
    return fetch("/api/billing");
  }
};
"#,
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "frontend/feature-system-adapter-contract"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("AbortSignal"))
    }));
}

#[test]
fn check_reports_feature_system_adapter_import_boundary_violations() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_adapter_contract_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-api.ts"),
        r#"import { BillingCard } from "../components/billing-card";
export class BillingApiError extends Error {}
export const billingApi = {
  async list(signal?: AbortSignal) {
    return fetch("/api/billing", { signal });
  },
  BillingCard
};
"#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/components/billing-card.tsx"),
        "export function BillingCard() { return null; }\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "frontend/feature-system-adapter-contract"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("adapter files may not import components"))
    }));
}

#[test]
fn check_accepts_custom_feature_system_adapter_contract_options() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_adapter_contract_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#"["error", {
      "systemsRoots": ["apps/web/src/modules"],
      "adapterDirectory": "api",
      "adapterFileNameTemplate": "{domain}.client.ts",
      "apiExportNameTemplate": "{domainCamel}Client",
      "errorExportNameTemplate": "{DomainPascal}ClientError",
      "httpClientNames": ["transport.get"]
    }]"#,
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/api/accounts.client.ts"),
        r#"export class AccountsClientError extends Error {}
export const accountsClient = {
  async list(signal?: AbortSignal) {
    return transport.get("/accounts", { signal });
  }
};
"#,
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_accepts_default_feature_system_query_contract() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_query_contract_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-api.ts"),
        "export const billingApi = { list: ({ signal }: { signal?: AbortSignal }) => fetch('/api/billing', { signal }), update: async () => undefined };\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/lib/query-keys.ts"),
        "export const billingQueryKeys = { list: () => ['billing'] as const };\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/lib/query-options.ts"),
        r#"import { queryOptions } from "@tanstack/react-query";
import { billingApi } from "../adapters/billing-api";
import { billingQueryKeys } from "./query-keys";
export const billingQueryOptions = () => queryOptions({
  queryKey: billingQueryKeys.list(),
  queryFn: ({ signal }) => billingApi.list({ signal }),
  staleTime: 60_000
});
"#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/hooks/use-billing.ts"),
        r#"import { useQuery } from "@tanstack/react-query";
import { billingQueryOptions } from "../lib/query-options";
export function useBilling() {
  return useQuery(billingQueryOptions());
}
"#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/hooks/use-update-billing.ts"),
        r#"import { useMutation, useQueryClient } from "@tanstack/react-query";
import { billingApi } from "../adapters/billing-api";
export function useUpdateBilling() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: billingApi.update,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["billing"] })
  });
}
export function useOptimisticBilling() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: billingApi.update,
    onMutate: async () => {
      await queryClient.cancelQueries({ queryKey: ["billing"] });
      const previous = queryClient.getQueryData(["billing"]);
      return { previous };
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey: ["billing"] })
  });
}
"#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/components/billing-card.tsx"),
        "import { useBilling } from '../hooks/use-billing';\nexport const BillingCard = useBilling;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_reports_missing_feature_system_query_files_and_inline_hooks() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_query_contract_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-api.ts"),
        "export const billingApi = { list: async () => [] };\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/hooks/use-billing.ts"),
        r#"import { useQuery } from "@tanstack/react-query";
import { billingApi } from "../adapters/billing-api";
export function useBilling() {
  return useQuery({
    queryKey: ["billing"],
    queryFn: () => billingApi.list()
  });
}
"#,
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "frontend/feature-system-query-contract"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("lib/query-keys.ts"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("lib/query-options.ts"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("query hooks should reuse factories"))
    }));
}

#[test]
fn check_reports_feature_system_query_option_shape_and_signal_violations() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_query_contract_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/adapters/billing-api.ts"),
        "export const billingApi = { list: async () => [] };\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/lib/query-keys.ts"),
        "export const billingQueryKeys = { list: () => ['billing'] as const };\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/lib/query-options.ts"),
        r#"import { queryOptions } from "@tanstack/react-query";
import { billingApi } from "../adapters/billing-api";
export const billingQueryOptions = () => queryOptions({
  queryFn: () => billingApi.list()
});
"#,
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("co-locate queryKey and queryFn"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("query context signal"))
    }));
}

#[test]
fn check_reports_component_and_route_owned_query_keys() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_query_contract_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/index.ts"),
        "export { BillingCard } from './components/billing-card';\n",
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/components/billing-card.tsx"),
        r#"import { useQuery } from "@tanstack/react-query";
export function BillingCard() {
  return useQuery({ queryKey: ["billing"], queryFn: async () => [] });
}
"#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/routes/billing-route.tsx"),
        r#"import { useQuery } from "@tanstack/react-query";
import { BillingCard } from "../systems/billing";
export const Route = () => useQuery({ queryKey: ["billing"], queryFn: async () => BillingCard });
"#,
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("components should not own query keys"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("routes should not own query keys"))
    }));
}

#[test]
fn check_reports_feature_system_mutation_cache_handling_violations() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_query_contract_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#""error""#,
    );
    write_file(
        &workspace
            .path()
            .join("packages/frontend/src/systems/billing/hooks/use-update-billing.ts"),
        r#"import { useMutation, useQueryClient } from "@tanstack/react-query";
export function useUpdateBilling() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async () => undefined,
    onMutate: () => {
      queryClient.setQueryData(["billing"], []);
    }
  });
}
"#,
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("invalidate relevant queries"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("optimistic cache updates"))
    }));
}

#[test]
fn check_accepts_custom_feature_system_query_contract_options() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_feature_system_query_contract_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        r#"["error", {
      "systemsRoots": ["apps/web/src/modules"],
      "routeRoots": ["apps/web/src/pages"],
      "adapterDirectory": "api",
      "queryKeysFile": "lib/keys.ts",
      "queryOptionsFile": "lib/options.ts"
    }]"#,
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/api/accounts-api.ts"),
        "export const accountsApi = { list: ({ signal }: { signal?: AbortSignal }) => fetch('/accounts', { signal }) };\n",
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/lib/keys.ts"),
        "export const accountKeys = { list: () => ['accounts'] as const };\n",
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/lib/options.ts"),
        r#"import { queryOptions } from "@tanstack/react-query";
import { accountsApi } from "../api/accounts-api";
import { accountKeys } from "./keys";
export const accountOptions = () => queryOptions({
  queryKey: accountKeys.list(),
  queryFn: ({ signal }) => accountsApi.list({ signal })
});
"#,
    );
    write_file(
        &workspace
            .path()
            .join("apps/web/src/modules/accounts/hooks/use-accounts.ts"),
        "import { useQuery } from '@tanstack/react-query';\nimport { accountOptions } from '../lib/options';\nexport const useAccounts = () => useQuery(accountOptions());\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_accepts_legacy_onion_rule_names_as_aliases() {
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
      "patterns": ["src/domain/**"],
      "mayImport": ["domain"]
    },
    "application": {
      "patterns": ["src/application/**"],
      "mayImport": ["application", "domain"]
    }
  },
  "rules": {
    "onion/no-layer-leak": "error"
  },
  "overrides": []
}"#,
    );
    write_file(
        &workspace.path().join("src/domain/order.ts"),
        r#"import { run } from "../application/use-case";
export const order = run;
"#,
    );
    write_file(
        &workspace.path().join("src/application/use-case.ts"),
        "export const run = () => undefined;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    assert_eq!(result["violations"][0]["rule"], "cleanarch/no-layer-leak");
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
    "cleanarch/no-layer-leak": "info"
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
                .and(predicate::str::contains("cleanarch/no-layer-leak"))
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
        violation["rule"] == "cleanarch/no-forbidden-imports"
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
        violation["rule"] == "cleanarch/no-cross-context-internal-import"
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
        violation["rule"] == "cleanarch/ambiguous-context"
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
fn check_enforces_architecture_specific_rules_not_delegated_to_oxlint() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_rules_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/sales/application/use-case.ts"),
        r#"import express from "express";
import { OrderRow } from "../infra/schemas/order.schema";
import { adapter } from "../infra/adapters/order.adapter";
import { BillingApi } from "../../billing/contracts/api";
import { CustomerSchema } from "../../billing/contracts/customer.schema";
export const run = [express, OrderRow, adapter, BillingApi, CustomerSchema];
"#,
    );
    write_file(
        &workspace.path().join("src/sales/application/envy.ts"),
        r#"import { BillingApi } from "../../billing/contracts/api";
import { BillingEvent } from "../../billing/events/created";
import { BillingPort } from "../../billing/ports/repository";
export const envy = [BillingApi, BillingEvent, BillingPort];
"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/sales/infra/schemas/order.schema.ts"),
        "export const OrderRow = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/sales/infra/adapters/order.adapter.ts"),
        "export const adapter = 1;\n",
    );
    write_file(
        &workspace.path().join("src/sales/contracts/index.ts"),
        r#"export { secret } from "../internal/model";
"#,
    );
    write_file(
        &workspace.path().join("src/sales/contracts/api.ts"),
        "export const SalesApi = 1;\n",
    );
    write_file(
        &workspace.path().join("src/sales/internal/model.ts"),
        "export const secret = 1;\n",
    );
    write_file(
        &workspace.path().join("src/billing/application/use-case.ts"),
        r#"import { SalesApi } from "../../sales/contracts/api";
export const bill = SalesApi;
"#,
    );
    write_file(
        &workspace.path().join("src/billing/contracts/api.ts"),
        "export const BillingApi = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/billing/contracts/customer.schema.ts"),
        "export const CustomerSchema = 1;\n",
    );
    write_file(
        &workspace.path().join("src/billing/events/created.ts"),
        "export const BillingEvent = 1;\n",
    );
    write_file(
        &workspace.path().join("src/billing/ports/repository.ts"),
        "export const BillingPort = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");
    let rules = violations
        .iter()
        .map(|violation| violation["rule"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();

    assert_eq!(result["status"], "fail");
    assert!(rules.contains(&"cleanarch/no-framework-in-core"));
    assert!(rules.contains(&"cleanarch/no-outer-data-format-in-core"));
    assert!(rules.contains(&"cleanarch/no-public-surface-internal-reexport"));
    assert!(rules.contains(&"cleanarch/no-context-cycle"));
    assert!(rules.contains(&"cleanarch/no-unowned-schema-import"));
    assert!(rules.contains(&"solid/no-concrete-dependency"));
    assert!(rules.contains(&"codesmells/feature-envy"));
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "cleanarch/no-context-cycle"
            && violation["cyclePath"]
                .as_array()
                .is_some_and(|cycle| cycle.first() == cycle.last())
    }));
}

#[test]
fn check_reports_shotgun_surgery_from_git_history_when_enabled() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_shotgun_policy_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(&workspace.path().join("src/a.ts"), "export const a = 0;\n");
    write_file(&workspace.path().join("src/b.ts"), "export const b = 0;\n");
    write_file(&workspace.path().join("src/c.ts"), "export const c = 0;\n");
    write_file(&workspace.path().join("src/d.ts"), "export const d = 0;\n");
    git(workspace.path(), &["init"]);
    git(workspace.path(), &["add", "."]);
    git(workspace.path(), &["commit", "-m", "initial"]);

    for index in 1..=2 {
        write_file(
            &workspace.path().join("src/a.ts"),
            &format!("export const a = {index};\n"),
        );
        write_file(
            &workspace.path().join("src/b.ts"),
            &format!("export const b = {index};\n"),
        );
        write_file(
            &workspace.path().join("src/c.ts"),
            &format!("export const c = {index};\n"),
        );
        git(workspace.path(), &["add", "."]);
        git(
            workspace.path(),
            &["commit", "-m", &format!("change {index}")],
        );
    }

    let result = run_json_check(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["warningCount"], 3);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "codesmells/shotgun-surgery"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("recurring companion files"))
    }));
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

    assert!(rules.contains(&"cleanarch/no-layer-leak"));
    assert!(rules.contains(&"cleanarch/no-cross-context-internal-import"));
    assert!(rules.contains(&"cleanarch/no-forbidden-imports"));
    assert!(!rules.contains(&"codesmells/unresolved-import"));
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
    assert_eq!(
        result["violations"][0]["rule"],
        "cleanarch/unclassified-file"
    );
}
