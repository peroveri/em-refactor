use assert_cmd::prelude::*;
use predicates::prelude::predicate;
use std::process::Command;
use tempdir::TempDir;

const WORKSPACE_ARG: &str = "--workspace-root=./tests/data/crates/hello_world";
const WORKSPACE_ARG_MULTI_ROOT: &str = "--workspace-root=./tests/data/crates/multi_root";

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

#[test]
fn provide_type() {
    let expected = r#"[{"type":"fn foo(i32,u32) -> (i32)"}]
"#;

    cargo_my_refactor()
        .arg(WORKSPACE_ARG)
        .arg("--provide-type")
        .arg("--selection=72:72")
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

#[test]
fn multiroot_project_lib() {

    let expected = r#"[{"file_name":"src/lib.rs","file_start_pos":0,"start":18,"end":21,"replacement":"Box<i32>"}]
"#;

    cargo_my_refactor()
        .arg(WORKSPACE_ARG_MULTI_ROOT)
        .arg("--output-changes-as-json")
        .arg("--ignore-missing-file")
        .arg("--refactoring=box-field")
        .arg("--selection=11:16")
        .arg("--file=src/lib.rs")
        .arg("--")
        .arg(format!(
            "--target-dir={}",
            create_tmp_dir().path().to_str().unwrap()
        ))
        .assert()
        .success()
        .stdout(expected);
}

#[test]
fn multiroot_project_main() {
    let expected = r#"[{"file_name":"src/main.rs","file_start_pos":0,"start":18,"end":21,"replacement":"Box<i32>"}]
"#;

    cargo_my_refactor()
        .arg(WORKSPACE_ARG_MULTI_ROOT)
        .arg("--output-changes-as-json")
        .arg("--ignore-missing-file")
        .arg("--refactoring=box-field")
        .arg("--selection=11:16")
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