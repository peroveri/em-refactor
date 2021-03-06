use assert_cmd::prelude::*;
use predicates::prelude::*;
use cli_tests_utils::*;
use em_refactor_lib_types::*;

mod cli_tests_utils;

mod cli_tests {
    use super::*;

    #[test]
    fn missing_args_should_output_nicely() {
        cargo_em_refactor()
            .arg(WORKSPACE_ARG)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("refactor")
            .assert()
            .failure()
            .stderr(predicate::str::starts_with(r#"error: The following required arguments were not provided:
    <refactoring>
    <file>
    <selection>"#));
    }

    #[test]
    fn multiroot_project_lib() {
        let expected = serde_json::to_string(
            &RefactorOutputs2::from_change(FileStringReplacement {
                byte_end: 21,
                byte_start: 18,
                char_end: 21,
                char_start: 18,
                file_name: "src/lib.rs".to_owned(),
                line_end: 0,
                line_start: 0,
                replacement: "Box<i32>".to_owned(),
        })).unwrap();

        cargo_em_refactor()
            .arg(WORKSPACE_ARG_MULTI_ROOT)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("refactor")
            .arg("box-field")
            .arg("src/lib.rs")
            .arg("11:16")
            .assert()
            .success()
            .stdout(expected);
    }

    #[test]
    fn multiroot_project_main() {
        let expected = serde_json::to_string(
            &RefactorOutputs2::from_change(FileStringReplacement {
                byte_end: 21,
                byte_start: 18,
                char_end: 21,
                char_start: 18,
                file_name: "src/main.rs".to_owned(),
                line_end: 0,
                line_start: 0,
                replacement: "Box<i32>".to_owned(),
        })).unwrap();

        cargo_em_refactor()
            .arg(WORKSPACE_ARG_MULTI_ROOT)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("refactor")
            .arg("box-field")
            .arg("src/main.rs")
            .arg("11:16")
            .assert()
            .success()
            .stdout(expected);
    }

    #[test]
    fn output_json() {
        let expected = serde_json::to_string(
            &RefactorOutputs2::from_change(FileStringReplacement {
                byte_end: 40,
                byte_start: 16,
                char_end: 28,
                char_start: 4,
                file_name: "src/main.rs".to_owned(),
                line_end: 1,
                line_start: 1,
                replacement: "let s = \n{let s = \"Hello, world!\";s};".to_owned(),
        })).unwrap();

        
        cargo_em_refactor()
            .arg(WORKSPACE_ARG)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("refactor")
            .arg("extract-block")
            .arg("src/main.rs")
            .arg("16:40")
            .assert()
            .success()
            .stdout(expected);
    }

    #[test]
    #[ignore]
    fn output_json_extract_method() {
        let expected = serde_json::to_string(
            &RefactorOutputs2::from_change(FileStringReplacement {
                byte_end: 101,
                byte_start: 100,
                char_end: 28,
                char_start: 6,
                file_name: "src/main.rs".to_owned(),
                line_end: 6,
                line_start: 6,
                replacement: "let s = \n{let s = \"Hello, world!\";s};".to_owned(),
        })).unwrap();

        
        cargo_em_refactor()
            .arg(WORKSPACE_ARG)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("refactor")
            .arg("extract-method")
            .arg("src/main.rs")
            .arg("100:101")
            .assert()
            .success()
            .stdout(expected);
    }

    #[test]
    fn output_json_rustc_codes() {
        let expected = serde_json::to_string(&RefactorOutputs2::from_error(
            RefactoringError {
                is_error: true,
                message: "error[E0597]: `i` does not live long enough\n --> src/main.rs:4:13\n  |\n2 |     let j = \n  |         - borrow later stored here\n3 | {let i = 0;\n4 |     let j = &i;j};\n  |             ^^  - `i` dropped here while still borrowed\n  |             |\n  |             borrowed value does not live long enough\n\n\nerror: aborting due to previous error\n\n\nFor more information about this error, try `rustc --explain E0597`.\n".to_owned(),
                kind: RefactorErrorType::RustCError2,
                codes: vec!["E0597".to_owned()],
                at_refactoring: "extract-block".to_owned()
            })).unwrap();
            
        cargo_em_refactor()
            .arg(WORKSPACE_ARG2)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("refactor")
            .arg("extract-block")
            .arg("src/main.rs")
            .arg("16:42")
            .assert()
            .success()
            .stdout(expected);
    }

    #[test]
    fn output_json_internal_codes() {
        let expected = serde_json::to_string(&RefactorOutputs2::from_error(
            RefactoringError {
                is_error: false,
                message: "0:1 is not a valid selection! `f`".to_owned(),
                kind: RefactorErrorType::Internal,
                codes: vec!["InvalidSelection".to_owned()],
                at_refactoring: "extract-block".to_owned()
            })).unwrap();
            
        cargo_em_refactor()
            .arg(WORKSPACE_ARG2)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("refactor")
            .arg("extract-block")
            .arg("src/main.rs")
            .arg("0:1")
            .assert()
            .success()
            .stdout(expected);
    }

    #[test]
    fn output_never() {
        let expected = serde_json::to_string(
            &RefactorOutputs2::from_error(RefactoringError {
                at_refactoring: "extract-block".to_owned(),
                codes: vec!["InvalidSelection".to_owned()],
                is_error: false,
                kind: RefactorErrorType::Internal,
                message: "97:107 is not a valid selection! `let x = 0;`".to_owned()
        })).unwrap();

        cargo_em_refactor()
            .arg(WORKSPACE_ARG2)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("refactor")
            .arg("extract-block")
            .arg("src/main.rs")
            .arg("97:107")
            .assert()
            .success()
            .stdout(expected);
    }

    #[test]
    fn query_candidates_1() {
        let expected = serde_json::to_string(
            &RefactorOutputs2::from_candidates(vec![
                CandidatePosition::new("src/main.rs", 16, 40, Some(1)),
                CandidatePosition::new("src/main.rs", 16, 63, Some(2)),
                CandidatePosition::new("src/main.rs", 45, 63, Some(1)),
                CandidatePosition::new("src/main.rs", 100, 101, Some(1)),
                CandidatePosition::new("src/main.rs", 124, 126, Some(1)),
        ])).unwrap();

        cargo_em_refactor()
            .arg(WORKSPACE_ARG)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("candidates")
            .arg("extract-method")
            .assert()
            .success()
            .stdout(expected);
    }

    #[test]
    fn query_candidates_on_invalid_crate() {
        cargo_em_refactor()
            .arg(WORKSPACE_ARG_INVALID_CRATE)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("candidates")
            .arg("extract-method")
            .assert()
            .failure()
            .stderr(predicate::str::starts_with("error: expected one of `("));
    }

    #[test]
    fn query_candidates_multi_root_overlap() {
        let expected = serde_json::to_string(&RefactorOutputs2::from_candidates(vec![
            CandidatePosition::new("src/lib.rs", 28, 41, Some(1)),
            CandidatePosition::new("src/main.rs", 29, 42, Some(1)),
            CandidatePosition::new("src/submod.rs", 47, 52, Some(1)),
        ])).unwrap();

        cargo_em_refactor()
            .arg(WORKSPACE_ARG_MULTI_ROOT_OVERLAP)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("candidates")
            .arg("extract-method")
            .assert()
            .success()
            .stdout(expected);
    }

    #[test]
    fn should_display_help() {
        cargo_em_refactor()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::starts_with(
            r#"Refactoring tool 0.0.1
Per Ove Ringdal <peroveri@gmail.com>

USAGE:
    cargo-em-refactor [FLAGS] [OPTIONS] <SUBCOMMAND>"#));
    }

    #[test]
    fn single_file() {
        let expected = 
r#"fn main() {
    let s = 
{let s = "Hello, world!";s};
    println!("{}", s);
}

fn foo(a: i32, b: u32) -> (i32) {1}

#[test]
fn test1() {2;}"#;

        cargo_em_refactor()
            .arg(WORKSPACE_ARG)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("--single-file")
            .arg("refactor")
            .arg("extract-block")
            .arg("src/main.rs")
            .arg("16:40")
            .assert()
            .success()
            .stdout(expected);
    }

    #[test]
    fn should_display_version() {
        cargo_em_refactor()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::starts_with("Refactoring tool 0.0.1"));
    }

    #[test]
    fn unknown_refactoring() {
        cargo_em_refactor()
            .arg(WORKSPACE_ARG)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("--single-file")
            .arg("refactor")
            .arg("invalid_refactoring_name")
            .arg("src/lib.rs")
            .arg("0:0")
            .assert()
            .failure()
            .stderr(predicate::str::starts_with(
                "Internal\nUnknown refactoring: invalid_refactoring_name",
            ));
    }

    #[test]
    fn workspace_deps() {
        cargo_em_refactor()
            .arg(WORKSPACE_DEPS_ARG)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("candidates")
            .arg("extract-method")
            .assert()
            .success();
    }

    #[test]
    fn workspace_no_deps() {
        cargo_em_refactor()
            .arg(WORKSPACE_NO_DEPS_ARG)
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .arg("candidates")
            .arg("extract-method")
            .assert()
            .success();
    }
}
