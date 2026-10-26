mod support;

use support::*;

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

const OVERRIDE_FAMILIES: &[&str] = &[
    "layout",
    "public-api",
    "dependency-flow",
    "adapter-contract",
    "query-contract",
];

fn write_feature_system_override_config(
    workspace: &TempDir,
    family: &str,
    setting: Value,
    overrides: Value,
) {
    let rule = format!("frontend/feature-system-{family}");
    let config = serde_json::json!({
        "version": 1,
        "project": { "root": ".", "include": ["**/*.ts", "**/*.tsx"], "exclude": [] },
        "rules": { (rule): setting },
        "overrides": overrides,
    });
    write_file(
        &workspace.path().join(".onioncryrc.jsonc"),
        &serde_json::to_string(&config).expect("config should serialize"),
    );
}

fn feature_system_override(family: &str, file: &str, setting: Value) -> Value {
    let rule = format!("frontend/feature-system-{family}");
    serde_json::json!({ "files": [file], "rules": { (rule): setting } })
}

// Each case has exactly one finding owned by the returned file under defaults.
fn write_feature_system_override_case(workspace: &TempDir, family: &str, name: &str) -> String {
    let root = "packages/frontend/src";
    let (file, source) = match family {
        "layout" => (
            format!("{root}/components/{name}/card.tsx"),
            "export const Card = () => null;".to_string(),
        ),
        "public-api" => {
            write_file(
                &workspace.path().join(format!("{root}/systems/billing/lib/internal.ts")),
                "export const internal = 1;",
            );
            (
                format!("{root}/routes/{name}.ts"),
                "import { internal } from '../systems/billing/lib/internal'; export const route = internal;".to_string(),
            )
        }
        "dependency-flow" => {
            write_file(
                &workspace.path().join(format!("{root}/systems/billing/adapters/billing-api.ts")),
                "export const billingApi = {};",
            );
            (
                format!("{root}/systems/billing/components/{name}.tsx"),
                "import { billingApi } from '../adapters/billing-api'; export const Card = billingApi;".to_string(),
            )
        }
        "adapter-contract" => (
            format!("{root}/systems/{name}/adapters/{name}-api.ts"),
            format!("export const {name}Api = {{}};"),
        ),
        "query-contract" => (
            format!("{root}/systems/billing/hooks/{name}.ts"),
            "import { useMutation } from '@tanstack/react-query'; export const mutate = () => useMutation({ mutationFn: async () => undefined });".to_string(),
        ),
        _ => panic!("unknown test family"),
    };
    write_file(&workspace.path().join(&file), &source);
    file
}

fn assert_feature_system_override_report(
    workspace: &TempDir,
    family: &str,
    expected: &[(&str, &str)],
    fail_on_warning: bool,
) -> Value {
    let errors = expected
        .iter()
        .filter(|(_, severity)| *severity == "error")
        .count();
    let warnings = expected.len() - errors;
    let fails = errors > 0 || (fail_on_warning && warnings > 0);
    let mut command = onioncry();
    command
        .current_dir(workspace.path())
        .args(["check", "--format", "json"]);
    if fail_on_warning {
        command.args(["--fail-on", "warning"]);
    }
    let output = command
        .assert()
        .code(if fails { 1 } else { 0 })
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&output.stdout).expect("check should emit JSON");
    assert_eq!(report["status"], if fails { "fail" } else { "pass" });
    assert_eq!(report["summary"]["errorCount"], errors);
    assert_eq!(report["summary"]["warningCount"], warnings);
    assert_eq!(report["summary"]["violationCount"], expected.len());
    let violations = report["violations"]
        .as_array()
        .expect("violations should be an array");
    let project_root = fs::canonicalize(workspace.path()).expect("workspace should resolve");
    let mut actual = violations
        .iter()
        .map(|violation| {
            assert_eq!(
                violation["rule"],
                format!("frontend/feature-system-{family}")
            );
            let file =
                std::path::Path::new(violation["file"].as_str().expect("file should be a path"));
            let relative = file.strip_prefix(&project_root).unwrap_or(file);
            (
                relative.to_string_lossy().replace('\\', "/"),
                violation["severity"]
                    .as_str()
                    .expect("severity should be a string")
                    .to_string(),
            )
        })
        .collect::<Vec<_>>();
    let mut expected = expected
        .iter()
        .map(|(file, severity)| (file.to_string(), severity.to_string()))
        .collect::<Vec<_>>();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected, "{family}: {report}");
    report
}

#[test]
fn feature_system_overrides_are_applied_per_diagnostic_file() {
    for family in OVERRIDE_FAMILIES {
        let workspace = TempDir::new().expect("workspace should be creatable");
        write_file(
            &workspace.path().join("aaa-unrelated.ts"),
            "export const unrelated = 1;",
        );
        let off = write_feature_system_override_case(&workspace, family, "off");
        let warn = write_feature_system_override_case(&workspace, family, "warn");
        let error = write_feature_system_override_case(&workspace, family, "error");
        let base = write_feature_system_override_case(&workspace, family, "base");
        let overrides = serde_json::json!([
            feature_system_override(family, "aaa-unrelated.ts", serde_json::json!("warn")),
            feature_system_override(family, &off, serde_json::json!("error")),
            feature_system_override(family, &off, serde_json::json!("off")),
            feature_system_override(family, &warn, serde_json::json!("off")),
            feature_system_override(family, &warn, serde_json::json!("warn")),
            feature_system_override(family, &error, serde_json::json!("warn")),
            feature_system_override(family, &error, serde_json::json!("error")),
        ]);
        write_feature_system_override_config(
            &workspace,
            family,
            serde_json::json!("error"),
            overrides.clone(),
        );
        let expected = [(&*warn, "warn"), (&*error, "error"), (&*base, "error")];
        assert_feature_system_override_report(&workspace, family, &expected, false);
        assert_feature_system_override_report(&workspace, family, &expected, true);

        // Enabling only the warning owner must preserve the default and strict exit contracts.
        let mut overrides = overrides
            .as_array()
            .expect("overrides should be an array")
            .clone();
        overrides.push(feature_system_override(
            family,
            &error,
            serde_json::json!("off"),
        ));
        write_feature_system_override_config(
            &workspace,
            family,
            serde_json::json!("off"),
            Value::Array(overrides),
        );
        assert_feature_system_override_report(&workspace, family, &[(&warn, "warn")], false);
        assert_feature_system_override_report(&workspace, family, &[(&warn, "warn")], true);
    }
}

#[test]
fn feature_system_overrides_replace_whole_options_per_file() {
    for family in OVERRIDE_FAMILIES {
        let workspace = TempDir::new().expect("workspace should be creatable");
        write_file(
            &workspace.path().join("aaa-unrelated.ts"),
            "export const unrelated = 1;",
        );
        let (owner, options) = if *family == "layout" {
            let root = "packages/frontend/src/systems/billing";
            write_file(
                &workspace.path().join(format!("{root}/index.ts")),
                "export const billing = 1;",
            );
            // An absent required directory is owned by the first file, index.ts.
            (
                format!("{root}/index.ts"),
                serde_json::json!({"requiredDirectories": ["custom"]}),
            )
        } else {
            let owner = write_feature_system_override_case(&workspace, family, "custom");
            let options = match *family {
                "public-api" => serde_json::json!({"routeRoots": []}),
                "dependency-flow" => serde_json::json!({"adapterBridgeFiles": []}),
                "adapter-contract" => serde_json::json!({"httpClientNames": []}),
                "query-contract" => serde_json::json!({"queryKeysFile": "lib/custom-keys.ts"}),
                _ => unreachable!(),
            };
            (owner, options)
        };
        write_feature_system_override_case(&workspace, family, "sibling");
        let base = if *family == "layout" {
            // Keep the sibling allowed outside systems as well.
            serde_json::json!(["error", {"systemsRoots": ["absent"], "allowedSharedComponentRoots": ["packages/frontend/src/components"]}])
        } else {
            serde_json::json!(["error", {"systemsRoots": ["absent"]}])
        };
        write_feature_system_override_config(
            &workspace,
            family,
            base,
            serde_json::json!([
                feature_system_override(family, &owner, serde_json::json!("off")),
                feature_system_override(family, &owner, serde_json::json!(["warn", options])),
            ]),
        );
        assert_feature_system_override_report(&workspace, family, &[(&owner, "warn")], false);
        assert_feature_system_override_report(&workspace, family, &[(&owner, "warn")], true);
    }
}

#[test]
fn feature_system_overrides_preserve_aggregate_owners() {
    for (family, owner, sibling, count) in [
        (
            "layout",
            "packages/frontend/src/systems/billing/a.ts",
            "packages/frontend/src/systems/billing/b.ts",
            3,
        ),
        (
            "adapter-contract",
            "packages/frontend/src/systems/billing/adapters/a.ts",
            "packages/frontend/src/systems/billing/adapters/b.ts",
            1,
        ),
        (
            "query-contract",
            "packages/frontend/src/systems/billing/a.ts",
            "packages/frontend/src/systems/billing/hooks/use-billing.ts",
            2,
        ),
    ] {
        let workspace = TempDir::new().expect("workspace should be creatable");
        write_file(
            &workspace.path().join("aaa-unrelated.ts"),
            "export const unrelated = 1;",
        );
        write_file(&workspace.path().join(owner), "export const a = 1;");
        write_file(
            &workspace.path().join(sibling),
            if family == "query-contract" {
                "import { useQuery } from '@tanstack/react-query'; export const useBilling = () => useQuery({ queryKey: ['billing'], queryFn: () => [] });"
            } else {
                "export const b = 1;"
            },
        );
        let sibling_setting = if family == "query-contract" {
            "off"
        } else {
            "error"
        };
        let overrides = serde_json::json!([
            feature_system_override(family, owner, serde_json::json!("warn")),
            feature_system_override(family, sibling, serde_json::json!(sibling_setting)),
        ]);
        write_feature_system_override_config(
            &workspace,
            family,
            serde_json::json!("error"),
            overrides.clone(),
        );
        let report = assert_feature_system_override_report(
            &workspace,
            family,
            &vec![(owner, "warn"); count],
            true,
        );
        let messages = report["violations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["message"].as_str().unwrap())
            .collect::<Vec<_>>();
        for message in match family {
            "layout" => vec![
                "missing required components/",
                "missing required lib/",
                "missing root index.ts",
            ],
            "adapter-contract" => vec!["adapter layer should expose adapters/billing-api.ts"],
            "query-contract" => vec![
                "requires lib/query-keys.ts",
                "requires lib/query-options.ts",
            ],
            _ => unreachable!(),
        } {
            assert!(
                messages.iter().any(|actual| actual.contains(message)),
                "{report}"
            );
        }
        let mut overrides = overrides.as_array().unwrap().clone();
        overrides.push(feature_system_override(
            family,
            owner,
            serde_json::json!("off"),
        ));
        write_feature_system_override_config(
            &workspace,
            family,
            serde_json::json!("error"),
            Value::Array(overrides),
        );
        assert_feature_system_override_report(&workspace, family, &[], true);
    }
}

#[test]
fn feature_system_overrides_keep_cross_file_query_and_adapter_facts() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    let root = "packages/frontend/src/systems/billing";
    let adapter = format!("{root}/adapters/billing-api.ts");
    let helper = format!("{root}/adapters/a-helper.ts");
    let keys = format!("{root}/lib/query-keys.ts");
    let options = format!("{root}/lib/query-options.ts");
    let hook = format!("{root}/hooks/use-billing.ts");
    write_file(
        &workspace.path().join(&adapter),
        "export const billingApi = { list: () => [] }; export class BillingApiError extends Error {}",
    );
    write_file(&workspace.path().join(&helper), "export const helper = 1;");
    write_file(
        &workspace.path().join(&keys),
        "export const billingKeys = { list: () => ['billing'] };",
    );
    write_file(
        &workspace.path().join(&options),
        "import { queryOptions } from '@tanstack/react-query'; import { billingApi } from '../adapters/billing-api'; import { billingKeys } from './query-keys'; export const billingOptions = () => queryOptions({ queryKey: billingKeys.list(), queryFn: ({ signal }) => billingApi.list({ signal }) });",
    );
    write_file(
        &workspace.path().join(&hook),
        "import { useQuery } from '@tanstack/react-query'; import { billingOptions } from '../lib/query-options'; export const useBilling = () => useQuery(billingOptions());",
    );
    write_feature_system_override_config(
        &workspace,
        "adapter-contract",
        serde_json::json!("off"),
        serde_json::json!([feature_system_override(
            "adapter-contract",
            &helper,
            serde_json::json!("warn")
        ),]),
    );
    assert_feature_system_override_report(&workspace, "adapter-contract", &[], true);
    write_feature_system_override_config(
        &workspace,
        "query-contract",
        serde_json::json!("off"),
        serde_json::json!([feature_system_override(
            "query-contract",
            &hook,
            serde_json::json!("warn")
        ),]),
    );
    assert_feature_system_override_report(&workspace, "query-contract", &[], true);

    // An adapter under Off still creates a cancellation obligation for the warning owner.
    write_file(
        &workspace.path().join(&options),
        "import { queryOptions } from '@tanstack/react-query'; import { billingApi } from '../adapters/billing-api'; export const billingOptions = () => queryOptions({ queryKey: ['billing'], queryFn: () => billingApi.list() });",
    );
    write_feature_system_override_config(
        &workspace,
        "query-contract",
        serde_json::json!("off"),
        serde_json::json!([feature_system_override(
            "query-contract",
            &options,
            serde_json::json!("warn")
        ),]),
    );
    let report = assert_feature_system_override_report(
        &workspace,
        "query-contract",
        &[(&options, "warn")],
        true,
    );
    assert!(
        report["violations"][0]["message"]
            .as_str()
            .unwrap()
            .contains("query context signal")
    );
}

#[test]
fn feature_system_overrides_exempt_export_hook_without_relaxing_production() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    let root = "packages/frontend/src/systems/billing";
    let exempt = format!("{root}/lib/export-hook.ts");
    let production = format!("{root}/components/production.tsx");
    let adapter = format!("{root}/adapters/billing-api.ts");
    write_file(
        &workspace.path().join(&exempt),
        "import { billingApi } from '../adapters/billing-api'; export const exportBilling = billingApi;",
    );
    write_file(
        &workspace.path().join(&production),
        "import { billingApi } from '../adapters/billing-api'; export const Production = billingApi;",
    );
    write_file(
        &workspace.path().join(&adapter),
        "export const billingApi = {};",
    );
    write_feature_system_override_config(
        &workspace,
        "dependency-flow",
        serde_json::json!("error"),
        serde_json::json!([feature_system_override(
            "dependency-flow",
            &exempt,
            serde_json::json!(["error", {"allowedImports": {"components": ["adapters"]}, "adapterBridgeFiles": ["lib/export-hook.ts"]}])
        ),]),
    );
    assert_feature_system_override_report(
        &workspace,
        "dependency-flow",
        &[(&production, "error")],
        true,
    );
}

const ERROR_EXPORT_ADAPTER: &str = "packages/frontend/src/systems/billing/adapters/billing-api.ts";

fn write_error_export_case(workspace: &TempDir, adapter: &str, modules: &[(&str, &str)]) {
    write_feature_system_override_config(
        workspace,
        "adapter-contract",
        serde_json::json!("error"),
        serde_json::json!([]),
    );
    write_file(
        &workspace.path().join(ERROR_EXPORT_ADAPTER),
        &format!("export const billingApi = {{}};\n{adapter}\n"),
    );
    for (file, source) in modules {
        write_file(&workspace.path().join(file), source);
    }
}

#[test]
fn feature_system_error_exports_follow_runtime_named_bindings() {
    let modules = [
        (
            "errors.ts",
            "export class ActualError extends Error {} export const value = 1;",
        ),
        (
            "hop.ts",
            "export { ActualError as Renamed } from './errors';",
        ),
        (
            "second.ts",
            "export { Renamed as FinalError } from './hop';",
        ),
        (
            "local.ts",
            "class InternalError extends Error {} export { InternalError as ActualError };",
        ),
        (
            "imported.ts",
            "import { ActualError as Internal } from './local'; export { Internal as Forwarded };",
        ),
        // Revisiting a file under a different binding is not a cycle.
        (
            "pair-a.ts",
            "export { Intermediate as ActualError } from './pair-b'; export class BaseError extends Error {}",
        ),
        (
            "pair-b.ts",
            "export { BaseError as Intermediate } from './pair-a';",
        ),
    ];
    for adapter in [
        "export class BillingApiError extends Error {}",
        "class BillingApiError extends Error {} export { BillingApiError };",
        "class InternalError extends Error {} export { InternalError as BillingApiError };",
        "export class BillingApiError\n extends\n Error {}",
        "export { ActualError as BillingApiError } from '../../../../../../errors';",
        "export {\n ActualError as\n BillingApiError,\n type value\n} from '../../../../../../errors';",
        "export { FinalError as BillingApiError } from '../../../../../../second';",
        "export { Forwarded as BillingApiError } from '../../../../../../imported';",
        "import { ActualError as LocalError } from '../../../../../../errors'; export { LocalError as BillingApiError };",
        "export { ActualError as BillingApiError } from '../../../../../../pair-a';",
    ] {
        let workspace = TempDir::new().expect("workspace should be creatable");
        write_error_export_case(&workspace, adapter, &modules);
        assert_feature_system_override_report(&workspace, "adapter-contract", &[], true);
    }
}

#[test]
fn feature_system_error_exports_reject_false_and_unknown_proofs() {
    let modules = [
        (
            "errors.ts",
            "export class ActualError extends Error {} export class Ordinary {} export const value = 1;",
        ),
        ("cycle-a.ts", "export { Loop as Again } from './cycle-b';"),
        ("cycle-b.ts", "export { Again as Loop } from './cycle-a';"),
        (
            "default.ts",
            "export default class ActualError extends Error {}",
        ),
        (
            "types.ts",
            "export declare class ActualError extends Error {}",
        ),
        ("ambient.d.ts", "export class ActualError extends Error {}"),
        (
            "type-export.ts",
            "export type { ActualError } from './errors';",
        ),
    ];
    for adapter in [
        "export class BillingApiError {} class Decoy extends Error {}",
        "class BillingApiError {} class Decoy extends Error {} export { BillingApiError };",
        "export const BillingApiError = 1; class Decoy extends Error {}",
        "// export class BillingApiError extends Error {}",
        "const text = 'export class BillingApiError extends Error {}';",
        "export class BillingApiErrorExtra extends Error {}",
        "class Decoy extends Error {} export { Missing as BillingApiError };",
        "class Base extends Error {} export class BillingApiError extends Base {}",
        "class BillingApiError extends Error {} export type { BillingApiError };",
        "class BillingApiError extends Error {} export { type BillingApiError };",
        "export declare class BillingApiError extends Error {}",
        "export { Ordinary as BillingApiError } from '../../../../../../errors';",
        "export { value as BillingApiError } from '../../../../../../errors';",
        "export { Missing as BillingApiError } from '../../../../../../errors';",
        "export type { ActualError as BillingApiError } from '../../../../../../errors';",
        "export { type ActualError as BillingApiError } from '../../../../../../errors';",
        "import type { ActualError as BillingApiError } from '../../../../../../errors'; export { BillingApiError };",
        "import { type ActualError as BillingApiError } from '../../../../../../errors'; export { BillingApiError };",
        "export { ActualError as BillingApiError } from '../../../../../../type-export';",
        "export { ActualError as BillingApiError } from '../../../../../../types';",
        "export { ActualError as BillingApiError } from '../../../../../../ambient.d';",
        "export { ActualError as BillingApiError } from 'external-errors';",
        "export { ActualError as BillingApiError } from './missing';",
        "export { Again as BillingApiError } from '../../../../../../cycle-a';",
        "export { BillingApiError } from './billing-api';",
        "export * from '../../../../../../errors';",
        "export { default as BillingApiError } from '../../../../../../default';",
        "import BillingApiError from '../../../../../../default'; export { BillingApiError };",
    ] {
        let workspace = TempDir::new().expect("workspace should be creatable");
        write_error_export_case(&workspace, adapter, &modules);
        let report = assert_feature_system_override_report(
            &workspace,
            "adapter-contract",
            &[(ERROR_EXPORT_ADAPTER, "error")],
            true,
        );
        assert_eq!(
            report["violations"][0]["message"],
            "adapter file should export typed API error BillingApiError",
            "{adapter}"
        );
    }
}

#[test]
fn feature_system_error_exports_do_not_read_excluded_targets() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_error_export_case(
        &workspace,
        "export { ActualError as BillingApiError } from '../../../../../../excluded/errors';",
        &[(
            "excluded/errors.ts",
            "export class ActualError extends Error {}",
        )],
    );
    let config_path = workspace.path().join(".onioncryrc.jsonc");
    let mut config: Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    config["project"]["exclude"] = serde_json::json!(["excluded/**"]);
    write_file(&config_path, &serde_json::to_string(&config).unwrap());
    for excluded_source in [
        "export class ActualError extends Error {}",
        "this is deliberately invalid TypeScript {",
    ] {
        write_file(
            &workspace.path().join("excluded/errors.ts"),
            excluded_source,
        );
        assert_feature_system_override_report(
            &workspace,
            "adapter-contract",
            &[(ERROR_EXPORT_ADAPTER, "error")],
            true,
        );
    }
}

#[test]
fn feature_system_error_exports_preserve_adapter_import_boundaries_and_strict_exits() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_error_export_case(
        &workspace,
        "export { ActualError as BillingApiError } from '../../../../../../errors'; import { Card } from '../components/card';",
        &[
            ("errors.ts", "export class ActualError extends Error {}"),
            (
                "packages/frontend/src/systems/billing/components/card.tsx",
                "export const Card = () => null;",
            ),
        ],
    );
    write_feature_system_override_config(
        &workspace,
        "adapter-contract",
        serde_json::json!("warn"),
        serde_json::json!([]),
    );
    for strict in [false, true] {
        let report = assert_feature_system_override_report(
            &workspace,
            "adapter-contract",
            &[(ERROR_EXPORT_ADAPTER, "warn")],
            strict,
        );
        assert!(
            report["violations"][0]["message"]
                .as_str()
                .unwrap()
                .contains("adapter files may not import components")
        );
        assert!(
            report["violations"][0]["targetFile"]
                .as_str()
                .unwrap()
                .ends_with("components/card.tsx")
        );
    }
}

#[test]
fn feature_system_error_exports_follow_configured_module_aliases() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_error_export_case(
        &workspace,
        "import { ActualError as Local } from '@errors/runtime'; export { Local as BillingApiError };",
        &[(
            "lib/runtime.ts",
            "export class ActualError extends Error {}",
        )],
    );
    let config_path = workspace.path().join(".onioncryrc.jsonc");
    let mut config: Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    config["aliases"] = serde_json::json!({"@errors": "lib"});
    write_file(&config_path, &serde_json::to_string(&config).unwrap());
    assert_feature_system_override_report(&workspace, "adapter-contract", &[], true);
}

#[test]
fn feature_system_error_exports_keep_shared_module_binding_identity() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_error_export_case(
        &workspace,
        "export { ActualError as BillingApiError } from '../../../../../../errors';",
        &[
            (
                "errors.ts",
                "export class ActualError extends Error {} export class Ordinary {}",
            ),
            (
                "packages/frontend/src/systems/orders/adapters/orders-api.ts",
                "export const ordersApi = {}; export { Ordinary as OrdersApiError } from '../../../../../../errors';",
            ),
        ],
    );
    assert_feature_system_override_report(
        &workspace,
        "adapter-contract",
        &[(
            "packages/frontend/src/systems/orders/adapters/orders-api.ts",
            "error",
        )],
        true,
    );
}

#[test]
fn feature_system_error_exports_reject_shadowed_error_superclasses() {
    for adapter in [
        "class Error {} export class BillingApiError extends Error {}",
        "const Error = class {}; export class BillingApiError extends Error {}",
        "import { Ordinary as Error } from '../../../../../../errors'; export class BillingApiError extends Error {}",
    ] {
        let workspace = TempDir::new().expect("workspace should be creatable");
        write_error_export_case(
            &workspace,
            adapter,
            &[("errors.ts", "export class Ordinary {}")],
        );
        assert_feature_system_override_report(
            &workspace,
            "adapter-contract",
            &[(ERROR_EXPORT_ADAPTER, "error")],
            true,
        );
    }
}
