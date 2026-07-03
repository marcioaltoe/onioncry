mod support;

use support::*;

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
