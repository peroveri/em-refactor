extern crate assert_cmd;

use assert_cmd::prelude::*;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

static TEST_CASE_PATH: &str = "../refactor-examples/extract_method";

fn read_test_file(file_path: &str) -> std::io::Result<String> {
    let mut path = PathBuf::from(TEST_CASE_PATH);
    path.push(file_path);
    let mut file = File::open(path)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    Ok(file_content)
}

fn json_str_to_param_vec(s: &str) -> serde_json::Result<Vec<String>> {
    let v: Value = serde_json::from_str(s)?;
    Ok(vec![
        format!("--refactoring={}", v["refactoring"].as_str().unwrap()),
        format!("--selection={}", v["selection"].as_str().unwrap()),
        format!("--new_function={}", v["new_function"].as_str().unwrap()),
    ])
}

fn run_testcase(name: &str, expect_sucess: bool) -> std::io::Result<()> {
    let refactoring_args = json_str_to_param_vec(&read_test_file(&format!("{}.json", name))?)?;
    let expected = if expect_sucess {read_test_file(&format!("{}_after.rs", name))?} else {"".to_string()};
    let assert = Command::cargo_bin("my-refactor-driver")
        .unwrap()
        .current_dir(TEST_CASE_PATH)
        .arg("--out-dir=../../tmp")
        .arg(format!("{}.rs", name))
        .arg("--")
        .args(refactoring_args)
        .arg(format!("--file={}.rs", name))
        .arg("--output-changes")
        .assert();

    if expect_sucess {
        assert.success().stdout(expected);
    } else {
        assert.failure();
    }
    Ok(())
}

fn run_test_and_assert_success(name: &str) {
    run_testcase(name, true).unwrap();
}
fn run_test_and_assert_failure(name: &str) {
    run_testcase(name, false).unwrap();
}

#[test]
fn extract_method_owned_mut_value() {
    run_test_and_assert_success("owned_mut_value");
}
#[test]
fn extract_method_borrowed_mut_value() {
    run_test_and_assert_success("borrowed_mut_value");
}
#[test]
fn extract_method_owned_value() {
    run_test_and_assert_success("owned_value");
}
#[test]
fn extract_method_failure_borrow_used_later() {
    run_test_and_assert_failure("failure_borrow_used_later");
}

// #[test]
fn nested_block() {
    run_test_and_assert_success("nested_block");
}

// #[test]
fn while_loop_inside() {
    run_test_and_assert_success("while_loop_inside");
}

#[test]
fn while_loop_outside() {
    run_test_and_assert_success("while_loop_outside");
}
