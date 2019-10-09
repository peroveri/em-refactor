extern crate assert_cmd;

use assert_cmd::prelude::*;
use std::process::Command;

static TEST_CASE_PATH: &str = "../refactor-examples/extract_method";

#[test]
fn missing_args_should_output_nicely() {
    Command::cargo_bin("my-refactor-driver")
        .unwrap()
        .current_dir(TEST_CASE_PATH)
        .arg("--out-dir=../../tmp")
        .arg("nested_block.rs")
        .arg("--")
        .assert()
        .code(3)
        .stderr("Expected --refactoring\n");
}

#[test]
fn unknown_refactoring() {
    Command::cargo_bin("my-refactor-driver")
        .unwrap()
        .current_dir(TEST_CASE_PATH)
        .arg("--out-dir=../../tmp")
        .arg("nested_block.rs")
        .arg("--")
        .arg("--refactoring=invalid_refactoring_name")
        .arg("--new_function=a")
        .arg("--selection=0:0")
        .assert()
        .code(3)
        .stderr("Unknown refactoring: invalid_refactoring_name\n");
}
