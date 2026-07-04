mod support;

use support::*;

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
fn check_accepts_custom_vertical_slice_public_surface_and_root_level_slices() {
    let custom_workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &custom_workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{
    "mode": "verticalSlice",
    "verticalSlice": {
      "sliceRoot": "modules",
      "sliceDepth": 1,
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
        "export const billing = 1;\nexport const setup = () => undefined;\n",
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
export const setup = () => undefined;
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
      "sliceDepth": 1,
      "allowedGlobalFolders": ["app", "config", "shared"]
    }
  }"#,
        r#"{
    "verticalslice/no-global-slice-artifacts": "error",
    "verticalslice/no-shared-layer-artifacts": "off"
  }"#,
        r#"[]"#,
    );
    write_file(
        &root_workspace
            .path()
            .join("src/orders/handlers/create-order.handler.ts"),
        "export const setup = () => undefined;\nexport const createOrder = () => undefined;\n",
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
    "verticalslice/no-global-slice-artifacts": "error",
    "verticalslice/no-shared-layer-artifacts": "off"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/features/orders/create-order/handlers/create-order.handler.ts"),
        "export const setup = () => undefined;\nexport const createOrder = () => undefined;\n",
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
    "verticalslice/no-global-slice-artifacts": "error",
    "verticalslice/no-shared-layer-artifacts": "off"
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
fn check_reports_vertical_slice_missing_configured_entry_point() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{ "mode": "verticalSlice" }"#,
        r#"{
    "verticalslice/slice-entry-point": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/features/orders/create-order/handlers/create-order.handler.ts"),
        "export const setupOrder = () => undefined;\nexport const createOrder = () => undefined;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["errorCount"], 1);
    assert_eq!(violations[0]["rule"], "verticalslice/slice-entry-point");
    assert!(violations[0]["message"].as_str().is_some_and(|message| {
        message.contains("orders/create-order") && message.contains("entry point")
    }));
}

#[test]
fn check_accepts_vertical_slice_configured_entry_point() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{ "mode": "verticalSlice" }"#,
        r#"{
    "verticalslice/slice-entry-point": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/features/orders/create-order/handlers/create-order.handler.ts"),
        "export const setup = () => undefined;\nexport const createOrder = () => undefined;\n",
    );

    let result = run_json_check(&workspace, &["check", "--format", "json"]);

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["violationCount"], 0);
}

#[test]
fn check_reports_vertical_slice_shared_layer_artifacts_outside_slices() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_architecture_mode_config(
        &workspace.path().join(".onioncryrc.jsonc"),
        "src",
        r#"{ "mode": "verticalSlice" }"#,
        r#"{
    "verticalslice/no-shared-layer-artifacts": "error"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/features/orders/create-order/repositories/order.repository.ts"),
        "export const setup = () => undefined;\nexport const orderRepository = 1;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/platform/repositories/order.repository.ts"),
        "export const platformRepository = 1;\n",
    );
    write_file(
        &workspace.path().join("src/shared/services/tax.service.ts"),
        "export const taxService = 1;\n",
    );

    let result = run_json_check_failure(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "fail");
    assert_eq!(result["summary"]["errorCount"], 2);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "verticalslice/no-shared-layer-artifacts"
            && violation["suggestion"]
                .as_str()
                .is_some_and(|suggestion| suggestion.contains("features/<domain>/<operation>"))
    }));
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
    "verticalslice/no-global-slice-artifacts": "off",
    "verticalslice/slice-entry-point": "off",
    "verticalslice/no-shared-layer-artifacts": "off"
  }"#,
        r#"[]"#,
    );
    write_file(
        &workspace
            .path()
            .join("src/features/billing/create-billing/handlers/create-billing.handler.ts"),
        "export const createBilling = () => undefined;\n",
    );
    write_file(
        &workspace
            .path()
            .join("src/features/orders/create-order/handlers/create-order.handler.ts"),
        "import { createBilling } from '../../../billing/create-billing/handlers/create-billing.handler';\nexport const createOrder = createBilling;\n",
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
