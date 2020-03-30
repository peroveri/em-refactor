use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use my_refactor_lib::*;

pub const WORKSPACE_ARG: &str = "--workspace-root=./tests/data/crates/hello_world";
pub const WORKSPACE_ARG_MULTI_ROOT: &str = "--workspace-root=./tests/data/crates/multi_root";
pub const WORKSPACE_ARG_MULTI_ROOT_OVERLAP: &str = "--workspace-root=./tests/data/crates/multi_root_overlap";

pub fn cargo_my_refactor() -> Command {
    Command::cargo_bin("cargo-my-refactor").unwrap()
}

pub fn create_tmp_dir() -> TempDir {
    let tmp_dir = TempDir::new().unwrap();
    let tmp_dir_path = tmp_dir.path();
    assert!(
        tmp_dir_path.is_dir(),
        "failed to create tmp dir: {}",
        tmp_dir_path.to_str().unwrap_or("")
    );
    tmp_dir
}

pub fn create_output(crate_name: &str, is_test: bool, replacement: &FileStringReplacement) -> RefactorOutput {
    RefactorOutput {
        crate_name: crate_name.to_owned(),
        is_test: is_test,
        replacements: vec![replacement.clone()],
        errors: vec![]
    }
}

pub fn create_output_err(crate_name: &str, is_test: bool, is_error: bool, message: &str) -> RefactorOutput {
    RefactorOutput {
        crate_name: crate_name.to_owned(),
        is_test: is_test,
        replacements: vec![],
        errors: vec![RefactoringError {
            is_error,
            message: message.to_string()
        }]
    }
}

pub fn assert_json_eq(expected: RefactorOutputs, actual: std::process::Output) {

    let out_str = String::from_utf8(actual.stdout.clone()).unwrap();

    actual.assert().success();
    
    let actual_json = serde_json::from_str::<RefactorOutputs>(&out_str).unwrap();

    assert_eq!(expected, actual_json);
}