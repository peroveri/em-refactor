use assert_cmd::prelude::*;
use std::process::Command;
use tempdir::TempDir;

pub const WORKSPACE_ARG: &str = "--workspace-root=./tests/data/crates/hello_world";
pub const WORKSPACE_ARG_MULTI_ROOT: &str = "--workspace-root=./tests/data/crates/multi_root";

pub fn cargo_my_refactor() -> Command {
    Command::cargo_bin("cargo-my-refactor").unwrap()
}

pub fn create_tmp_dir() -> TempDir {
    let tmp_dir = TempDir::new("my_refactoring_tool").unwrap();
    let tmp_dir_path = tmp_dir.path();
    assert!(
        tmp_dir_path.is_dir(),
        "failed to create tmp dir: {}",
        tmp_dir_path.to_str().unwrap_or("")
    );
    tmp_dir
}

pub fn create_output(crate_name: &str, is_test: bool, replacement: &FileReplaceContent) -> RefactorOutput {
    RefactorOutput {
        crate_name: crate_name.to_owned(),
        is_test: is_test,
        replacements: vec![replacement.clone()],
        errors: vec![]
    }
}

pub fn assert_json_eq(expected: Vec<RefactorOutput>, actual: std::process::Output) {

    let out_str = String::from_utf8(actual.stdout.clone()).unwrap();

    actual.assert().success();


    let mut list = vec![];
    for line in out_str.split("\n") {
        if line.trim().len() == 0 {
            continue;
        }
        assert!(line.starts_with("Crate:"), format!("{}", line));
        let json_str = &line["Crate:".len()..];
        let out_json = serde_json::from_str::<RefactorOutput>(json_str);
        list.push(out_json.unwrap());
    }
    
    assert_eq!(expected, list);
}

// These structs are copied from src/change.rs
// Should import them instead
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FileReplaceContent {
    pub byte_end: u32,
    pub byte_start: u32,
    pub char_end: usize,
    pub char_start: usize,
    pub file_name: String,
    pub line_end: usize,
    pub line_start: usize,
    pub replacement: String
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RefactoringError {
    pub byte_end: u32,
    pub byte_start: u32,
    pub char_end: usize,
    pub char_start: usize,
    pub file_name: String,
    pub line_end: usize,
    pub line_start: usize,
    pub message: String
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RefactorOutput {
    pub crate_name: String,
    // pub root_path: String,
    pub is_test: bool,
    pub replacements: Vec<FileReplaceContent>,
    pub errors: Vec<RefactoringError>
}