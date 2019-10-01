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
        .success()
        .stdout("Expected --refactoring\n");
}
