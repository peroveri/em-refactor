use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::json;
use cli_tests_utils::*;

mod cli_tests_utils;

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
fn cli_multiroot_project_lib() {
    let replacement = FileReplaceContent {
        byte_end: 21,
        byte_start: 18,
        char_end: 21,
        char_start: 18,
        file_name: "src/lib.rs".to_owned(),
        line_end: 0,
        line_start: 0,
        replacement: "Box<i32>".to_owned(),
    };
    let expected = vec![
        create_output("lib", false, &replacement),
        create_output("lib", true, &replacement),
        create_output_err("main", false, false, "Couldn't find file: src/lib.rs"),
        create_output_err("main", true, false, "Couldn't find file: src/lib.rs"),
    ];

    let actual = cargo_my_refactor()
        .arg(WORKSPACE_ARG_MULTI_ROOT)
        .arg("--output-replacements-as-json")
        .arg("--refactoring=box-field")
        .arg("--selection=11:16")
        .arg("--file=src/lib.rs")
        .arg("--")
        .arg(format!(
            "--target-dir={}",
            create_tmp_dir().path().to_str().unwrap()
        )).output()
        .unwrap();

    assert_json_eq(expected, actual);
}

#[test]
fn cli_multiroot_project_main() {
    let replacement = FileReplaceContent {
        byte_end: 21,
        byte_start: 18,
        char_end: 21,
        char_start: 18,
        file_name: "src/main.rs".to_owned(),
        line_end: 0,
        line_start: 0,
        replacement: "Box<i32>".to_owned(),
    };
    let expected = vec![
        create_output_err("lib", false, false, "Couldn't find file: src/main.rs"),
        create_output_err("lib", true, false, "Couldn't find file: src/main.rs"),
        create_output("main", false, &replacement),
        create_output("main", true, &replacement),
    ];

    let actual = cargo_my_refactor()
        .arg(WORKSPACE_ARG_MULTI_ROOT)
        .arg("--output-replacements-as-json")
        .arg("--refactoring=box-field")
        .arg("--selection=11:16")
        .arg("--file=src/main.rs")
        .arg("--")
        .arg(format!(
            "--target-dir={}",
            create_tmp_dir().path().to_str().unwrap()
        )).output().unwrap();

    assert_json_eq(expected, actual);
}

#[test]
fn cli_output_json() {
    let replacement = FileReplaceContent {
        byte_end: 40,
        byte_start: 16,
        char_end: 28,
        char_start: 4,
        file_name: "src/main.rs".to_owned(),
        line_end: 1,
        line_start: 1,
        replacement: "let s = \n{\nlet s = \"Hello, world!\";\ns};".to_owned(),
    };
    let expected = vec![
        create_output("hello_world", false, &replacement),
        create_output("hello_world", true, &replacement),
    ];

    let actual = cargo_my_refactor()
        .arg(WORKSPACE_ARG)
        .arg("--output-replacements-as-json")
        .arg("--refactoring=extract-block")
        .arg("--selection=16:40")
        .arg("--file=src/main.rs")
        .arg("--")
        .arg(format!(
            "--target-dir={}",
            create_tmp_dir().path().to_str().unwrap()
        ))
        .output().unwrap();
    
    assert_json_eq(expected, actual);
}

#[test]
fn cli_provide_type() {
    let expected = format!("{}\n{}\n", json!([{
        "type": "fn foo(i32,u32) -> (i32)"
    }]), json!([{
        "type": "fn foo(i32,u32) -> (i32)"
    }]));

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
fn cli_query_candidates_1() {
    let expected_main = format!("{}", json!({
        "candidates": [{
            "file": "src/main.rs",
            "from": 16,
            "to": 40
        },{
            "file": "src/main.rs",
            "from": 16,
            "to": 63
        },{
            "file": "src/main.rs",
            "from": 45,
            "to": 63
        },{
            "file": "src/main.rs",
            "from": 100,
            "to": 101
        }],
        "refactoring": "extract-block"
    }));
    let expected = format!("{}\n{}\n", expected_main, expected_main);

    cargo_my_refactor()
        .arg(WORKSPACE_ARG)
        .arg("--query-candidates=extract-block")
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
fn cli_query_candidates_multi_root_overlap() {
    let expected_lib = format!("{}", json!({
        "candidates": [{
            "file": "src/submod.rs",
            "from": 47,
            "to": 52
        },{
            "file": "src/lib.rs",
            "from": 28,
            "to": 41
        }],
        "refactoring": "extract-block"
    }));
    let expected_main = format!("{}", json!({
        "candidates": [{
            "file": "src/submod.rs",
            "from": 47,
            "to": 52
        },{
            "file": "src/main.rs",
            "from": 29,
            "to": 42
        }],
        "refactoring": "extract-block"
    }));
    let expected = format!("{}\n{}\n{}\n{}\n", expected_lib, expected_lib, expected_main, expected_main);

    cargo_my_refactor()
        .arg(WORKSPACE_ARG_MULTI_ROOT_OVERLAP)
        .arg("--query-candidates=extract-block")
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
