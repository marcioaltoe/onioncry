mod support;

use support::*;

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
      "sliceDepth": 2,
      "publicSurface": ["index.ts", "contracts"],
      "artifactFolders": ["handlers", "adapters", "domain", "__tests__"],
      "artifactSuffixes": {
        "handler": [".handler.ts"],
        "service": [".service.ts"]
      },
      "allowedGlobalFolders": ["app", "config", "lib", "shared", "platform"]
    }
  }"#,
        r#"{}"#,
        r#"[]"#,
    );
    write_file(
        &vertical_workspace
            .path()
            .join("src/features/orders/list-orders/index.ts"),
        "export const orders = 1;\n",
    );

    let vertical_result = run_json_check(&vertical_workspace, &["check", "--format", "json"]);
    assert_eq!(vertical_result["status"], "pass");
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
