use assert_cmd::prelude::*;
use std::process::Command;

static TEST_CASE_PATH: &str = "../refactor-examples/extract_block";
static TEST_PROJECT_PATH: &str = "../refactor-examples/project";

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

#[test]
fn output_json() {
    let expected = r#"[{"file_name":"extract_block_1.rs","file_start_pos":0,"start":31,"end":62,"replacement":"let i = \n{\nlet i = 1;\n    print!(\"{}\", i);\ni};"}]"#;

    Command::cargo_bin("my-refactor-driver")
        .unwrap()
        .current_dir(TEST_CASE_PATH)
        .arg("--out-dir=../../tmp")
        .arg("extract_block_1.rs")
        .arg("--")
        .arg("--output-changes-as-json")
        .arg("--refactoring=extract-block")
        .arg("--selection=31:62")
        .arg("--file=extract_block_1.rs")
        .assert()
        .code(0)
        .stdout(expected);
}

// #[test]
// fn project_example() {
//     Command::new("touch -c src/mod1.rs").current_dir(TEST_PROJECT_PATH).spawn().unwrap();
//     Command::cargo_bin("cargo-my-refactor")
//         .unwrap()
//         .current_dir(TEST_PROJECT_PATH)
//         .arg("--")
//         .arg("--")
//         .arg("--out-dir=../../tmp")
//         .arg("--")
//         .arg("--refactoring=box-field")
//         .arg("--selection=18:19")
//         .arg("--file=src/mod1.rs")
//         .arg("--output-changes-as-json")
//         .assert()
//         .code(0)
//         .stdout("asd");
// }

#[test]
fn project_example_extract_block() {
    Command::cargo_bin("cargo-my-refactor")
        .unwrap()
        .current_dir(TEST_PROJECT_PATH)
        .arg("--")
        .arg("--")
        // .arg("--out-dir=../../tmp")
        .arg("--")
        .arg("--refactoring=extract-block")
        .arg("--selection=42:52")
        .arg("--file=src/mod1.rs")
        .assert()
        .code(0);
}

