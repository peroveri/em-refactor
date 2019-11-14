use assert_cmd::prelude::*;
use predicates::prelude::predicate;
use std::process::Command;

fn cargo_my_refactor() -> Command {
    Command::cargo_bin("cargo-my-refactor").unwrap()
}

fn touch_main_rs() {
    Command::new("touch")
        .arg("-c")
        .arg("/home/perove/dev/github.uio.no/refactor-rust/tmp/hello_world/src/main.rs")
        .spawn()
        .unwrap();
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
fn cli_missing_args_should_output_nicely() {
    touch_main_rs();
    cargo_my_refactor()
        .arg("--workspace-root=../tmp/hello_world")
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("Expected --refactoring\n"));
}

#[test]
fn cli_unknown_refactoring() {
    touch_main_rs();
    cargo_my_refactor()
        .arg("--workspace-root=../tmp/hello_world")
        .arg("--refactoring=invalid_refactoring_name")
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("Unknown refactoring: invalid_refactoring_name\n"));
}

#[test]
fn cli_output_json() {
    touch_main_rs();
    let expected = r#"[{"file_name":"src/main.rs","file_start_pos":0,"start":16,"end":26,"replacement":"{\nlet i = 0;\n}"}]
"#;

    cargo_my_refactor()
        .arg("--workspace-root=../tmp/hello_world")
        .arg("--output-changes-as-json")
        .arg("--refactoring=extract-block")
        .arg("--selection=16:26")
        .arg("--file=src/main.rs")
        .assert()
        .success()
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
