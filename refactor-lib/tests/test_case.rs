use assert_cmd::prelude::*;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

// These tests are currently not thread safe for multiple tests on a single .rs file
// run single threaded with: cargo test -- --test-threads=1

static TEST_CASE_PATH: &str = "../refactor-examples";

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
        let args_serde = &v["args"];
        let mut args = vec![
            format!("--refactoring={}", args_serde["refactoring"].as_str().unwrap()),
            format!("--selection={}", args_serde["selection"].as_str().unwrap()),
        ];
        if let Some(new_fn) = args_serde["new_function"].as_str() {
            args.push(format!("--new_function={}", new_fn))
        }
        Ok(args)
    }
}

fn read_test_file(folder: &str, file_name: &str) -> std::io::Result<String> {
    let path: PathBuf = [TEST_CASE_PATH, folder, file_name].iter().collect();
    let mut file = File::open(path)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    Ok(file_content)
}

pub fn run_testcase(folder: &str, name: &str) -> std::io::Result<()> {
    let json_content = read_test_file(folder, &format!("{}.json", name))?;
    let mut test = TestCase::from_json(&json_content)?;
    if let Some(ref f) = test.expected.stdout_file {
        // read expected output from file if set
        test.expected.stdout = Some(read_test_file(folder, &f)?);
    }
    run_tool_and_assert(test, folder)
}

fn run_tool_and_assert(test: TestCase, folder: &str) -> std::io::Result<()> {
    let path: PathBuf = [TEST_CASE_PATH, folder].iter().collect();
    let mut assert = Command::cargo_bin("my-refactor-driver")
        .unwrap()
        .current_dir(path)
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
