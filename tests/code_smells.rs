mod support;

use support::*;

#[test]
fn check_reports_shotgun_surgery_from_git_history_when_enabled() {
    let workspace = TempDir::new().expect("workspace should be creatable");
    write_shotgun_policy_config(&workspace.path().join(".onioncryrc.jsonc"));
    write_file(&workspace.path().join("src/a.ts"), "export const a = 0;\n");
    write_file(&workspace.path().join("src/b.ts"), "export const b = 0;\n");
    write_file(&workspace.path().join("src/c.ts"), "export const c = 0;\n");
    write_file(&workspace.path().join("src/d.ts"), "export const d = 0;\n");
    git(workspace.path(), &["init"]);
    git(workspace.path(), &["add", "."]);
    git(workspace.path(), &["commit", "-m", "initial"]);

    for index in 1..=2 {
        write_file(
            &workspace.path().join("src/a.ts"),
            &format!("export const a = {index};\n"),
        );
        write_file(
            &workspace.path().join("src/b.ts"),
            &format!("export const b = {index};\n"),
        );
        write_file(
            &workspace.path().join("src/c.ts"),
            &format!("export const c = {index};\n"),
        );
        git(workspace.path(), &["add", "."]);
        git(
            workspace.path(),
            &["commit", "-m", &format!("change {index}")],
        );
    }

    let result = run_json_check(&workspace, &["check", "--format", "json"]);
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");

    assert_eq!(result["status"], "pass");
    assert_eq!(result["summary"]["warningCount"], 3);
    assert!(violations.iter().all(|violation| {
        violation["rule"] == "codesmells/shotgun-surgery"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("recurring companion files"))
    }));
}
