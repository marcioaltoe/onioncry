mod support;

use support::*;

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
