// Each integration test crate compiles this shared module independently, so helpers
// used by sibling test crates can look unused from one crate's point of view.
#![allow(dead_code, unused_imports)]

use assert_cmd::Command;
pub use predicates::prelude::*;
pub use serde_json::Value;
pub use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
pub use tempfile::TempDir;

pub fn onioncry() -> Command {
    Command::cargo_bin("onioncry").expect("onioncry binary should be built for tests")
}

pub fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test parent directory should be creatable");
    }
    fs::write(path, contents).expect("test file should be writable");
}

pub fn git(workspace: &Path, args: &[&str]) {
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

pub fn assert_llm_report_metadata_line(metadata: &str) {
    let revision = metadata
        .strip_prefix("onioncry-llm-report v1 revision: ")
        .expect("llm report metadata should include a revision");

    assert!(!revision.is_empty());
    assert!(!revision.contains("buildTimestamp"));
}

pub fn strip_full_line_jsonc_comments(contents: &str) -> String {
    contents
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn write_minimal_config(path: &Path, root: &str, include: &[&str], exclude: &[&str]) {
    write_config(path, root, include, exclude, "{}");
}

pub fn write_config(
    path: &Path,
    root: &str,
    include: &[&str],
    exclude: &[&str],
    aliases_json: &str,
) {
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

pub fn write_layer_config(path: &Path) {
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

pub fn write_rule_policy_config(path: &Path) {
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

pub fn write_external_package_policy_config(path: &Path) {
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

pub fn write_context_policy_config(path: &Path) {
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

pub fn write_architecture_rules_config(path: &Path) {
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

pub fn write_architecture_mode_config(
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

pub fn write_shotgun_policy_config(path: &Path) {
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

pub fn write_test_placement_config(path: &Path, rule_json: &str, overrides_json: &str) {
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

pub fn write_path_naming_config(path: &Path, rule_json: &str) {
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

pub fn write_feature_system_layout_config(path: &Path, rule_json: &str) {
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

pub fn write_feature_system_public_api_config(path: &Path, rule_json: &str) {
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

pub fn write_feature_system_dependency_flow_config(path: &Path, rule_json: &str) {
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

pub fn write_feature_system_adapter_contract_config(path: &Path, rule_json: &str) {
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

pub fn write_feature_system_query_contract_config(path: &Path, rule_json: &str) {
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

pub fn write_explain_config(path: &Path) {
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

pub fn run_json_check(workspace: &TempDir, args: &[&str]) -> Value {
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

pub fn run_json_explain(workspace: &TempDir, args: &[&str]) -> Value {
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

pub fn run_json_check_failure(workspace: &TempDir, args: &[&str]) -> Value {
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
