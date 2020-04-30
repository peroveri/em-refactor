use assert_cmd::prelude::*;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// These tests are currently not thread safe for multiple tests on a single .rs file
// run single threaded with: cargo test -- --test-threads=1

static TEST_CASE_PATH: &str = "../refactor-examples";

struct TestCase {
    file: String,
    refactoring: String,
    selection: String,
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
        let args = TestCase::json_str_to_param_vec(&v);
        Ok(TestCase {
            file: v["file"].as_str().unwrap().to_string(),
            refactoring: args.0,
            selection: args.1,
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
    fn json_str_to_param_vec(v: &Value) -> (String, String) {
        let args = &v["args"];
        (args["refactoring"].as_str().unwrap().to_string(), args["selection"].as_str().unwrap().to_string())
    }
}

fn read_test_file(folder: &str, file_name: &str) -> std::io::Result<String> {
    let path: PathBuf = [TEST_CASE_PATH, folder, file_name].iter().collect();
    assert!(
        path.is_file(),
        "path should be a file, but isn't: {}",
        path.to_str().unwrap_or("")
    );
    let mut file = File::open(path)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    Ok(file_content)
}

fn write_test_file(file_path: &PathBuf, content: &str) -> std::io::Result<()> {
    assert!(
        file_path.is_file(),
        "path should be a file, but isn't: {}",
        file_path.to_str().unwrap_or("")
    );
    let mut file = File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    file.flush()?;
    Ok(())
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
    let content = read_test_file(folder, &test.file)?;
    let tmp_dir = TempDir::new()?;
    let tmp_dir_path = tmp_dir.path();
    assert!(
        tmp_dir_path.is_dir(),
        "failed to create tmp dir: {}",
        tmp_dir_path.to_str().unwrap_or("")
    );
    Command::new("cargo")
        .current_dir(tmp_dir_path)
        .arg("init")
        .arg("--name=test_case")
        .assert().success();

    let main_rs_path = tmp_dir_path.join("src").join("main.rs");
    write_test_file(&main_rs_path, &content)?;
    
    let mut assert = Command::cargo_bin("cargo-my-refactor")
        .unwrap()
        .arg(format!("--workspace-root={}", tmp_dir_path.to_str().unwrap()))
        .arg(format!("--target-dir={}", tmp_dir_path.join("target").join("refactorings").to_str().unwrap()))
        .arg("--single-file")
        .arg("refactor")
        .arg(&test.refactoring)
        .arg("src/main.rs")
        .arg(&test.selection)
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
