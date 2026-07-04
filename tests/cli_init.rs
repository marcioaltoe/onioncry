mod support;

use support::*;

#[test]
fn cli_prints_version() {
    onioncry().arg("--version").assert().success().stdout(
        predicate::str::contains(format!("onioncry {}\n", onioncry::CLI_VERSION))
            .and(predicate::str::contains("buildTimestamp").not()),
    );
}

#[test]
fn init_creates_parseable_mvp_template() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    onioncry()
        .current_dir(workspace.path())
        .args(["init"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".onioncryrc.jsonc"));

    let config_path = workspace.path().join(".onioncryrc.jsonc");
    let config = fs::read_to_string(&config_path).expect("init config should be readable");

    assert!(config.contains(
        r#""$schema": "https://raw.githubusercontent.com/marcioaltoe/onioncry/main/docs/schema/onioncryrc.schema.json""#
    ));
    assert!(config.contains(r#""version""#));
    assert!(config.contains(r#""project""#));
    assert!(config.contains(r#""architecture""#));
    assert!(config.contains(r#""mode": "cleanArchitecture""#));
    assert!(config.contains(r#""cleanArchitecture""#));
    assert!(config.contains(r#""verticalSlice""#));
    assert!(config.contains(r#""contextRoot": "contexts""#));
    assert!(config.contains(r#""sliceRoot": "features""#));
    assert!(config.contains(r#""layerPathAliases""#));
    assert!(config.contains(r#""artifactFolders""#));
    assert!(config.contains(r#""artifactSuffixes""#));
    assert!(config.contains(r#""groupedArtifactFolders""#));
    assert!(config.contains(r#""aliases""#));
    assert!(config.contains(r#""layers""#));
    assert!(config.contains(r#""contexts""#));
    assert!(config.contains(r#""contextRules""#));
    assert!(config.contains(r#""rules""#));
    assert!(config.contains(r#""overrides""#));
    assert!(config.contains("TODO"));
    assert!(config.contains(r#""domain""#));
    assert!(config.contains(r#""application""#));
    assert!(config.contains(r#""infra""#));
    assert!(config.contains(r#""shared""#));
    assert!(config.contains(r#""mayImport": ["domain", "shared"]"#));
    assert!(config.contains(r#""cleanarch/no-layer-leak": "error""#));
    assert!(config.contains(r#""cleanarch/no-cross-context-internal-import": "error""#));
    assert!(config.contains(r#""fromLayer": "domain""#));
    assert!(config.contains(r#""severity": "error""#));
    assert!(config.contains(r#""fromLayer": "application""#));
    assert!(config.contains(r#""severity": "warn""#));
    assert!(config.contains(r#""fromLayer": "infra""#));
    assert!(config.contains(r#""severity": "off""#));
    assert!(!config.contains("codesmells/unresolved-import"));
    assert!(!config.contains("codesmells/circular-dependency"));
    assert!(config.contains(r#""cleanarch/no-framework-in-core": "warn""#));
    assert!(config.contains(r#""cleanarch/no-outer-data-format-in-core": "warn""#));
    assert!(config.contains(r#""cleanarch/no-public-surface-internal-reexport": "warn""#));
    assert!(config.contains(r#""cleanarch/no-context-cycle": "warn""#));
    assert!(config.contains(r#""cleanarch/no-unowned-schema-import": "warn""#));
    assert!(config.contains(r#""cleanarch/artifact-placement": "warn""#));
    assert!(config.contains(r#""solid/no-concrete-dependency": "warn""#));
    assert!(config.contains(r#""codesmells/feature-envy": "warn""#));
    assert!(config.contains(r#""codesmells/shotgun-surgery": "off""#));
    assert!(config.contains(r#""cleanarch/unclassified-file": "warn""#));
    assert!(config.contains(r#""verticalslice/no-cross-slice-internal-import": "warn""#));
    assert!(config.contains(r#""verticalslice/no-global-slice-artifacts": "warn""#));
    assert!(config.contains(r#""sliceDepth": 2"#));
    assert!(config.contains(r#""platform""#));
    assert!(config.contains(r#""entryPointNames""#));
    assert!(config.contains(r#""sharedLayerFolders""#));
    assert!(config.contains(r#""verticalslice/slice-entry-point": "warn""#));
    assert!(config.contains(r#""verticalslice/no-shared-layer-artifacts": "warn""#));

    let stripped = strip_full_line_jsonc_comments(&config);
    let parsed: Value =
        serde_json::from_str(&stripped).expect("template should parse after comments are stripped");
    assert_eq!(parsed["version"], 1);
    assert!(parsed["project"].is_object());
    assert_eq!(parsed["architecture"]["mode"], "cleanArchitecture");
    assert_eq!(
        parsed["architecture"]["cleanArchitecture"]["contextRoot"],
        "contexts"
    );
    assert_eq!(
        parsed["architecture"]["cleanArchitecture"]["groupedArtifactFolders"][0],
        "use-cases"
    );
    assert!(
        parsed["architecture"]["cleanArchitecture"]["groupedArtifactFolders"]
            .as_array()
            .expect("grouped artifact folders should be an array")
            .iter()
            .any(|folder| folder == "repositories")
    );
    assert!(
        parsed["architecture"]["cleanArchitecture"]["groupedArtifactFolders"]
            .as_array()
            .expect("grouped artifact folders should be an array")
            .iter()
            .any(|folder| folder == "bootstrap")
    );
    assert_eq!(
        parsed["architecture"]["verticalSlice"]["sliceRoot"],
        "features"
    );
    assert_eq!(parsed["architecture"]["verticalSlice"]["sliceDepth"], 2);
    assert!(parsed["aliases"].is_object());
    assert!(parsed["layers"].is_object());
    assert!(parsed["contexts"].is_object());
    assert!(parsed["contextRules"].is_object());
    assert!(parsed["rules"].is_object());
    assert!(parsed["overrides"].is_array());
}

#[test]
fn init_keeps_the_default_alias_block_without_from_tsconfig() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    onioncry()
        .current_dir(workspace.path())
        .args(["init"])
        .assert()
        .success();

    let config = fs::read_to_string(workspace.path().join(".onioncryrc.jsonc"))
        .expect("init config should be readable");
    assert!(config.contains(r#""@app/": "src/""#));
    assert!(!config.contains("Aliases generated from"));
}

#[test]
fn init_from_tsconfig_generates_reviewable_aliases_and_lists_skipped_entries() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_file(
        &workspace.path().join("tsconfig.json"),
        r#"{
  // JSONC comments are valid in tsconfig files.
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"],
      "@shared/*": ["./src/shared/*"],
      "helpers": ["./src/helpers/index.ts"],
      "@multi/*": ["./a/*", "./b/*"],
      "@out/*": ["../outside/*"]
    }
  }
}"#,
    );

    onioncry()
        .current_dir(workspace.path())
        .args(["init", "--from-tsconfig"])
        .assert()
        .success();

    let config = fs::read_to_string(workspace.path().join(".onioncryrc.jsonc"))
        .expect("init config should be readable");
    assert!(config.contains("Aliases generated from tsconfig.json for review"));
    assert!(config.contains(r#""@/": "src/""#));
    assert!(config.contains(r#""@shared/": "src/shared/""#));
    assert!(!config.contains(r#""@app/": "src/""#));
    assert!(config.contains("Skipped tsconfig paths entries (map these manually):"));
    assert!(config.contains(r#""helpers": non-wildcard key"#));
    assert!(config.contains(r#""@multi/*": multiple targets"#));
    assert!(config.contains(r#""@out/*": target outside the project root"#));

    let stripped = strip_full_line_jsonc_comments(&config);
    let parsed: Value = serde_json::from_str(&stripped)
        .expect("generated template should parse after comments are stripped");
    assert_eq!(parsed["aliases"]["@/"], "src/");
    assert_eq!(parsed["aliases"]["@shared/"], "src/shared/");
    assert_eq!(
        parsed["aliases"]
            .as_object()
            .expect("aliases should be an object")
            .len(),
        2
    );
}

#[test]
fn init_from_tsconfig_accepts_an_explicit_path_and_notes_extends() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_file(
        &workspace.path().join("packages/app/tsconfig.json"),
        r#"{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@app/*": ["./src/*"]
    }
  }
}"#,
    );

    onioncry()
        .current_dir(workspace.path())
        .args(["init", "--from-tsconfig", "packages/app/tsconfig.json"])
        .assert()
        .success();

    let config = fs::read_to_string(workspace.path().join(".onioncryrc.jsonc"))
        .expect("init config should be readable");
    assert!(config.contains("Aliases generated from packages/app/tsconfig.json for review"));
    assert!(config.contains(r#"Note: this tsconfig uses "extends", which init does not follow."#));
    assert!(config.contains(r#""@app/": "packages/app/src/""#));

    onioncry()
        .current_dir(workspace.path())
        .args([
            "init",
            "--force",
            "--from-tsconfig",
            "packages/app/tsconfig.json",
        ])
        .assert()
        .success();
}

#[test]
fn init_from_tsconfig_reports_errors_without_leaving_a_config_behind() {
    let workspace = TempDir::new().expect("workspace should be creatable");

    onioncry()
        .current_dir(workspace.path())
        .args(["init", "--from-tsconfig"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("tsconfig does not exist"));
    assert!(!workspace.path().join(".onioncryrc.jsonc").exists());

    write_file(&workspace.path().join("tsconfig.json"), "{ not valid jsonc");
    onioncry()
        .current_dir(workspace.path())
        .args(["init", "--from-tsconfig"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("could not parse tsconfig"));
    assert!(!workspace.path().join(".onioncryrc.jsonc").exists());
}

#[test]
fn init_does_not_overwrite_existing_config_unless_forced() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    let config_path = workspace.path().join(".onioncryrc.jsonc");
    write_file(&config_path, "existing config\n");

    onioncry()
        .current_dir(workspace.path())
        .args(["init"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains(".onioncryrc.jsonc").and(predicate::str::contains("--force")),
        );
    assert_eq!(
        fs::read_to_string(&config_path).expect("existing config should be readable"),
        "existing config\n"
    );

    onioncry()
        .current_dir(workspace.path())
        .args(["init", "--force"])
        .assert()
        .success();

    let forced = fs::read_to_string(&config_path).expect("forced config should be readable");
    assert!(forced.contains(r#""cleanarch/no-layer-leak": "error""#));
    assert_ne!(forced, "existing config\n");
}
