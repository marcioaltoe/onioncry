mod support;

use std::path::Path;
use support::*;

// Suite: schema command
// Invariant: onioncry schema emits the same JSON Schema that is committed for editor tooling.
// Boundary IN: public onioncry CLI output, file writes, and committed schema artifact.
// Boundary OUT: JSON Schema validator behavior in external editors.

#[test]
fn schema_prints_configuration_schema_without_config() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    let schema = run_json_schema(&workspace, &["schema"]);

    assert_eq!(schema["title"], "Config");
    assert!(
        schema["$schema"]
            .as_str()
            .expect("schema should declare a draft")
            .contains("json-schema.org")
    );
    assert!(schema["properties"]["project"].is_object());
    assert!(schema["properties"]["architecture"].is_object());
    assert!(schema["properties"]["rules"].is_object());
    assert!(schema["properties"]["overrides"].is_object());

    let schema_text =
        serde_json::to_string(&schema).expect("schema should serialize for contract checks");
    assert!(schema_text.contains("cleanArchitecture"));
    assert!(schema_text.contains("verticalSlice"));
    assert!(schema_text.contains("mayImport"));
    assert!(schema_text.contains("default"));
    assert!(schema_text.contains("files"));
}

#[test]
fn schema_write_creates_requested_file_and_prints_path() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    let output_path = workspace.path().join("generated/onioncryrc.schema.json");

    let output = onioncry()
        .current_dir(workspace.path())
        .args(["schema", "--write", "generated/onioncryrc.schema.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("schema --write output should be utf-8");

    assert!(stdout.contains("created generated/onioncryrc.schema.json"));
    assert!(output_path.exists());

    let written_schema: Value =
        serde_json::from_slice(&fs::read(&output_path).expect("written schema should be readable"))
            .expect("written schema should be valid JSON");
    let stdout_schema = run_json_schema(&workspace, &["schema"]);
    assert_eq!(written_schema, stdout_schema);
}

#[test]
fn committed_schema_matches_generated_schema() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let committed_schema_path = repo_root.join("docs/schema/onioncryrc.schema.json");
    let generated_schema = run_json_schema_at(repo_root, &["schema"]);
    let committed_schema: Value = serde_json::from_slice(
        &fs::read(&committed_schema_path).expect("committed schema should be readable"),
    )
    .expect("committed schema should be valid JSON");

    assert_eq!(
        committed_schema, generated_schema,
        "docs/schema/onioncryrc.schema.json is stale; run `onioncry schema --write docs/schema/onioncryrc.schema.json`"
    );
}

fn run_json_schema(workspace: &TempDir, args: &[&str]) -> Value {
    run_json_schema_at(workspace.path(), args)
}

fn run_json_schema_at(workspace: &Path, args: &[&str]) -> Value {
    let output = onioncry()
        .current_dir(workspace)
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).expect("schema command should emit valid JSON")
}
