mod support;

use support::*;

#[test]
fn graph_renders_clean_architecture_contexts_as_mermaid() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_context_graph_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/sales/internal/use-case.ts"),
        r#"import { invoice } from "../../billing/contracts/invoice";
export const run = invoice;
"#,
    );
    write_file(
        &workspace.path().join("src/billing/contracts/invoice.ts"),
        "export const invoice = 1;\n",
    );

    let output = onioncry()
        .current_dir(workspace.path())
        .args(["graph"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let mermaid = String::from_utf8(output).expect("graph output should be utf-8");

    assert_eq!(
        mermaid,
        "graph TD\n  n0[\"billing\"]\n  n1[\"sales\"]\n  n1 -->|contracts| n0\n"
    );
}

#[test]
fn graph_renders_clean_architecture_contexts_as_json() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_context_graph_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/sales/internal/use-case.ts"),
        r#"import { invoice } from "../../billing/contracts/invoice";
import { invoice as otherInvoice } from "../../billing/contracts/invoice";
export const run = invoice + otherInvoice;
"#,
    );
    write_file(
        &workspace.path().join("src/billing/contracts/invoice.ts"),
        "export const invoice = 1;\n",
    );

    let graph = run_json_graph(&workspace);

    assert_eq!(graph["nodes"][0]["id"], "billing");
    assert_eq!(graph["nodes"][0]["kind"], "context");
    assert_eq!(graph["nodes"][1]["id"], "sales");
    assert_eq!(graph["edges"][0]["from"], "sales");
    assert_eq!(graph["edges"][0]["to"], "billing");
    assert_eq!(graph["edges"][0]["via"], "contracts");
    assert_eq!(graph["edges"][0]["importCount"], 2);
}

#[test]
fn graph_renders_vertical_slices_as_mermaid() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_vertical_graph_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace
            .path()
            .join("features/orders/internal/use-case.ts"),
        r#"import { customer } from "../../customers/index";
export const run = customer;
"#,
    );
    write_file(
        &workspace.path().join("features/customers/index.ts"),
        "export const customer = 1;\n",
    );

    let output = onioncry()
        .current_dir(workspace.path())
        .args(["graph"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let mermaid = String::from_utf8(output).expect("graph output should be utf-8");

    assert_eq!(
        mermaid,
        "graph TD\n  n0[\"customers\"]\n  n1[\"orders\"]\n  n1 -->|index.ts| n0\n"
    );
}

#[test]
fn graph_renders_vertical_slices_as_json() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_vertical_graph_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace
            .path()
            .join("features/orders/internal/use-case.ts"),
        r#"import { customer } from "../../customers/index";
export const run = customer;
"#,
    );
    write_file(
        &workspace.path().join("features/customers/index.ts"),
        "export const customer = 1;\n",
    );

    let graph = run_json_graph(&workspace);

    assert_eq!(graph["nodes"][0]["id"], "customers");
    assert_eq!(graph["nodes"][0]["kind"], "slice");
    assert_eq!(graph["nodes"][1]["id"], "orders");
    assert_eq!(graph["edges"][0]["from"], "orders");
    assert_eq!(graph["edges"][0]["to"], "customers");
    assert_eq!(graph["edges"][0]["via"], "index.ts");
    assert_eq!(graph["edges"][0]["importCount"], 1);
}

#[test]
fn graph_handles_empty_projects() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_context_graph_config(&workspace.path().join(".onioncryrc.jsonc"));

    let graph = run_json_graph(&workspace);

    assert_eq!(
        graph["nodes"]
            .as_array()
            .expect("nodes should be an array")
            .len(),
        0
    );
    assert_eq!(
        graph["edges"]
            .as_array()
            .expect("edges should be an array")
            .len(),
        0
    );
}

#[test]
fn graph_reports_config_errors_with_exit_code_two() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    onioncry()
        .current_dir(workspace.path())
        .args(["graph"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(".onioncryrc"));
}

#[test]
fn graph_aggregates_contextless_files_into_a_single_node() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_context_graph_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(
        &workspace.path().join("src/tools/report.ts"),
        r#"import { invoice } from "../billing/contracts/invoice";
export const report = invoice;
"#,
    );
    write_file(
        &workspace.path().join("src/tools/export.ts"),
        r#"import { invoice } from "../billing/contracts/invoice";
export const exported = invoice;
"#,
    );
    write_file(
        &workspace.path().join("src/billing/contracts/invoice.ts"),
        "export const invoice = 1;\n",
    );

    let graph = run_json_graph(&workspace);
    let nodes = graph["nodes"].as_array().expect("nodes should be an array");

    assert_eq!(
        nodes
            .iter()
            .filter(|node| node["kind"] == "contextless")
            .count(),
        1
    );
    assert!(nodes.iter().any(|node| node["id"] == "contextless"));
    assert_eq!(graph["edges"][0]["from"], "contextless");
    assert_eq!(graph["edges"][0]["to"], "billing");
    assert_eq!(graph["edges"][0]["via"], "contracts");
    assert_eq!(graph["edges"][0]["importCount"], 2);
}

fn run_json_graph(workspace: &TempDir) -> Value {
    let output = onioncry()
        .current_dir(workspace.path())
        .args(["graph", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).expect("graph --format json should emit JSON")
}

fn write_context_graph_config(path: &std::path::Path) {
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
      "allowCrossContext": ["contracts"]
    }
  },
  "rules": {},
  "overrides": []
}"#,
    );
}

fn write_vertical_graph_config(path: &std::path::Path) {
    write_file(
        path,
        r#"{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["features/**/*.ts"],
    "exclude": []
  },
  "architecture": {
    "mode": "verticalSlice",
    "verticalSlice": {
      "sliceRoot": "features",
      "sliceDepth": 1,
      "publicSurface": ["index.ts"]
    }
  },
  "rules": {},
  "overrides": []
}"#,
    );
}
