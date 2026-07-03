mod support;

use support::*;

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
        &workspace
            .path()
            .join("src/features/billing/create-billing/index.ts"),
        "export const billing = 1;\nexport const setup = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/features/billing/create-billing/contracts/billing-event.ts"),
        "export type BillingEvent = { id: string };\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/features/billing/create-billing/handlers/create-billing.handler.ts"),
        "export type BillingHandler = () => void;\nexport const createBilling = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/features/orders/create-order/domain/order.service.ts"),
        "export const orderService = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/features/orders/create-order/handlers/create-order.handler.ts"),
        r#"import { billing } from "../../../billing/create-billing";
import type { BillingEvent } from "../../../billing/create-billing/contracts/billing-event";
import { orderService } from "../domain/order.service";
import type { BillingHandler } from "../../../billing/create-billing/handlers/create-billing.handler";
export { createBilling } from "../../../billing/create-billing/handlers/create-billing.handler";
export const setup = () => undefined;
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
                .is_some_and(|suggestion| suggestion.contains("features/billing/create-billing"))
    }));
    assert!(violations.iter().all(|violation| {
        violation["importSpecifier"]
            == "../../../billing/create-billing/handlers/create-billing.handler"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("orders/create-order"))
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("billing/create-billing"))
    }));
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
