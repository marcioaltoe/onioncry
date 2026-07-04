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
