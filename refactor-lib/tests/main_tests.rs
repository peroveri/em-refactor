extern crate assert_cmd;

use assert_cmd::prelude::*;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

static TEST_CASE_PATH: &str = "../refactor-examples/extract_method_01/src";

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

fn run_testcase(name: &str) -> std::io::Result<()> {
    let expected = read_test_file(&format!("{}_after.rs", name))?;
    let refactoring_args = json_str_to_param_vec(&read_test_file(&format!("{}.json", name))?)?;
    Command::cargo_bin("my-refactor-driver")
        .unwrap()
        .current_dir(TEST_CASE_PATH)
        .arg("--out-dir=../../../tmp")
        .arg(format!("{}.rs", name))
        .arg("--")
        .args(refactoring_args)
        .arg(format!("--file={}.rs", name))
        .arg("--output-changes")
        .assert()
        .success()
        .stdout(expected);
    Ok(())
}

#[test]
fn extract_method_owned_mut_value() {
    run_testcase("owned_mut_value").unwrap();
}
#[test]
fn extract_method_borrowed_mut_value() {
    run_testcase("borrowed_mut_value").unwrap();
}
#[test]
fn extract_method_owned_value() {
    run_testcase("owned_value").unwrap();
}
