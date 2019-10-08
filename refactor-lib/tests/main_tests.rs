extern crate assert_cmd;

use assert_cmd::prelude::*;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

static TEST_CASE_PATH: &str = "../refactor-examples/extract_method";

struct TestCase {
    file: String,
    args: Vec<String>,
    expected: Expected,
}
/**
 * Assertions are only made for fields with values
 */
struct Expected {
    code: Option<i64>,
    stderr: Option<String>,
    stdout: Option<String>,
    stdout_file: Option<String>,
}

impl TestCase {
    fn from_json(s: &str) -> serde_json::Result<TestCase> {
        let v: Value = serde_json::from_str(s)?;
        Ok(TestCase {
            file: v["file"].as_str().unwrap().to_string(),
            args: TestCase::json_str_to_param_vec(&v)?,
            expected: TestCase::map_expected(&v).unwrap(),
        })
    }
    fn map_expected(v: &Value) -> Option<Expected> {
        let expected = &v["expected"];
        Some(Expected {
            code: expected["code"].as_i64(),
            stderr: expected["stderr"].as_str().map(|s| s.to_string()),
            stdout: expected["stdout"].as_str().map(|s| s.to_string()),
            stdout_file: expected["stdout_file"].as_str().map(|s| s.to_string()),
        })
    }
    fn json_str_to_param_vec(v: &Value) -> serde_json::Result<Vec<String>> {
        let args = &v["args"];
        Ok(vec![
            format!("--refactoring={}", args["refactoring"].as_str().unwrap()),
            format!("--selection={}", args["selection"].as_str().unwrap()),
            format!("--new_function={}", args["new_function"].as_str().unwrap()),
        ])
    }
}

fn read_test_file(file_path: &str) -> std::io::Result<String> {
    let mut path = PathBuf::from(TEST_CASE_PATH);
    path.push(file_path);
    let mut file = File::open(path)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    Ok(file_content)
}

fn run_testcase(name: &str) -> std::io::Result<()> {
    let json_content = read_test_file(&format!("{}.json", name))?;
    let mut test = TestCase::from_json(&json_content)?;
    if let Some(ref f) = test.expected.stdout_file {
        // read expected output from file if set
        test.expected.stdout = Some(read_test_file(&f)?);
    }
    run_tool_and_assert(test)
}

fn run_tool_and_assert(test: TestCase) -> std::io::Result<()> {
    let mut assert = Command::cargo_bin("my-refactor-driver")
        .unwrap()
        .current_dir(TEST_CASE_PATH)
        .arg("--out-dir=../../tmp")
        .arg(&test.file)
        .arg("--")
        .args(test.args)
        .arg(format!("--file={}", &test.file))
        .arg("--output-changes")
        .assert();

    if let Some(code) = test.expected.code {
        assert = assert.code(code as i32);
    }
    if let Some(ref stdout) = test.expected.stdout {
        assert = assert.stdout(stdout.to_string());
    }
    if let Some(ref stderr) = test.expected.stderr {
        assert.stderr(stderr.to_string());
    }
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
#[test]
fn extract_method_failure_borrow_used_later() {
    run_testcase("failure_borrow_used_later").unwrap();
}
#[test]
fn nested_block() {
    run_testcase("nested_block").unwrap();
}
#[test]
fn while_loop_inside() {
    run_testcase("while_loop_inside").unwrap();
}
#[test]
fn while_loop_outside() {
    run_testcase("while_loop_outside").unwrap();
}
#[test]
fn failure_selection_break_id() {
    run_testcase("failure_selection_break_id").unwrap();
}
#[test]
fn failure_selection_empty() {
    run_testcase("failure_selection_empty").unwrap();
}
#[test]
fn failure_selection_unbalanced() {
    run_testcase("failure_selection_unbalanced").unwrap();
}
