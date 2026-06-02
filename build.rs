use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src");
    emit_git_rerun_hints();

    println!(
        "cargo:rustc-env=ONIONCRY_BUILD_REVISION={}",
        current_git_revision()
    );
}

fn emit_git_rerun_hints() {
    let Some(git_dir) = git_dir() else {
        println!("cargo:rerun-if-changed=.git");
        return;
    };

    println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
    println!("cargo:rerun-if-changed={}", git_dir.join("index").display());
    println!(
        "cargo:rerun-if-changed={}",
        git_dir.join("packed-refs").display()
    );

    let head = git_dir.join("HEAD");
    if let Ok(contents) = fs::read_to_string(head)
        && let Some(reference) = contents.trim().strip_prefix("ref: ")
    {
        println!(
            "cargo:rerun-if-changed={}",
            git_dir.join(reference).display()
        );
    }
}

fn git_dir() -> Option<PathBuf> {
    let dot_git = Path::new(".git");
    if dot_git.is_dir() {
        return Some(dot_git.to_path_buf());
    }

    let contents = fs::read_to_string(dot_git).ok()?;
    let path = contents.trim().strip_prefix("gitdir:")?.trim();
    let git_dir = PathBuf::from(path);
    Some(if git_dir.is_absolute() {
        git_dir
    } else {
        dot_git
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(git_dir)
    })
}

fn current_git_revision() -> String {
    let Some(revision) = command_stdout("git", &["rev-parse", "--short=12", "HEAD"]) else {
        return "unknown".to_string();
    };

    if has_tracked_git_changes() {
        format!("{revision}-dirty")
    } else {
        revision
    }
}

fn has_tracked_git_changes() -> bool {
    Command::new("git")
        .args(["diff", "--quiet", "HEAD", "--"])
        .status()
        .map(|status| !status.success())
        .unwrap_or(false)
}

fn command_stdout(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}
