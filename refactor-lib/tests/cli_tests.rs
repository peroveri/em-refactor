use assert_cmd::prelude::*;
use predicates::prelude::predicate;
use std::process::Command;
use tempdir::TempDir;

const WORKSPACE_ARG: &str = "--workspace-root=./tests/data/crates/hello_world";

fn cargo_my_refactor() -> Command {
    Command::cargo_bin("cargo-my-refactor").unwrap()
}

fn create_tmp_dir() -> TempDir {
    let tmp_dir = TempDir::new("my_refactoring_tool").unwrap();
    let tmp_dir_path = tmp_dir.path();
    assert!(
        tmp_dir_path.is_dir(),
        "failed to create tmp dir: {}",
        tmp_dir_path.to_str().unwrap_or("")
    );
    tmp_dir
}

#[test]
fn cli_should_display_help() {
    cargo_my_refactor()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "Refactorings for the Rust programming language.",
        ));
}
#[test]
fn cli_should_display_version() {
    cargo_my_refactor()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("Version:"));
}

#[test]
fn cli_missing_args_should_output_nicely() {
    cargo_my_refactor()
        .arg(WORKSPACE_ARG)
        .arg("--")
        .arg(format!(
            "--target-dir={}",
            create_tmp_dir().path().to_str().unwrap()
        ))
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("Expected --refactoring\n"));
}

#[test]
fn cli_unknown_refactoring() {
    cargo_my_refactor()
        .arg(WORKSPACE_ARG)
        .arg("--refactoring=invalid_refactoring_name")
        .arg("--")
        .arg(format!(
            "--target-dir={}",
            create_tmp_dir().path().to_str().unwrap()
        ))
        .assert()
        .failure()
        .stderr(predicate::str::starts_with(
            "Unknown refactoring: invalid_refactoring_name\n",
        ));
}

#[test]
fn cli_output_json() {
    let expected = r#"[{"file_name":"src/main.rs","file_start_pos":0,"start":16,"end":40,"replacement":"let s = \n{\nlet s = \"Hello, world!\";\ns};"}]
"#;

    cargo_my_refactor()
        .arg(WORKSPACE_ARG)
        .arg("--output-changes-as-json")
        .arg("--refactoring=extract-block")
        .arg("--selection=16:40")
        .arg("--file=src/main.rs")
        .arg("--")
        .arg(format!(
            "--target-dir={}",
            create_tmp_dir().path().to_str().unwrap()
        ))
        .assert()
        .success()
        .stdout(expected);
}
