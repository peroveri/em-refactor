use assert_cmd::prelude::*;
use predicates::prelude::*;
use cli_tests_utils::*;
use refactor_lib::*;

mod cli_tests_utils;

#[test]
fn cli_missing_args_should_output_nicely() {
    cargo_my_refactor()
        .arg(WORKSPACE_ARG)
        .arg("--selection=0:0")
        .arg("--file=main.rs")
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
    let replacement = FileStringReplacement {
        byte_end: 21,
        byte_start: 18,
        char_end: 21,
        char_start: 18,
        file_name: "src/lib.rs".to_owned(),
        line_end: 0,
        line_start: 0,
        replacement: "Box<i32>".to_owned(),
    };
    let expected = RefactorOutputs {
        refactorings: vec![
            create_output("lib", false, &replacement),
            create_output("lib", true, &replacement),
            create_output_err("main", false, false, "Couldn't find file: src/lib.rs"),
            create_output_err("main", true, false, "Couldn't find file: src/lib.rs"),
        ],
        candidates: vec![]
    };

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
    let replacement = FileStringReplacement {
        byte_end: 21,
        byte_start: 18,
        char_end: 21,
        char_start: 18,
        file_name: "src/main.rs".to_owned(),
        line_end: 0,
        line_start: 0,
        replacement: "Box<i32>".to_owned(),
    };
    let expected = RefactorOutputs {
        refactorings: vec![
            create_output_err("lib", false, false, "Couldn't find file: src/main.rs"),
            create_output_err("lib", true, false, "Couldn't find file: src/main.rs"),
            create_output("main", false, &replacement),
            create_output("main", true, &replacement),
        ],
        candidates: vec![]
    };

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
    let replacement = FileStringReplacement {
        byte_end: 40,
        byte_start: 16,
        char_end: 28,
        char_start: 4,
        file_name: "src/main.rs".to_owned(),
        line_end: 1,
        line_start: 1,
        replacement: "let s = \n{let s = \"Hello, world!\";s};".to_owned(),
    };
    let expected =  RefactorOutputs {
        refactorings: vec![
            create_output("hello_world", false, &replacement),
            create_output("hello_world", true, &replacement),
        ],
        candidates: vec![]
    };

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
fn cli_query_candidates_1() {
    let file = "src/main.rs";
    let map_position = |from, to| CandidatePosition { file: file.to_string(), from, to};
    let candidates = vec![
        map_position(16, 40),
        map_position(16, 63),
        map_position(45, 63),
        map_position(100, 101),
    ];
    let candidates_test = candidates.clone().into_iter().chain(vec![
        map_position(124, 126),
    ].into_iter()).collect::<Vec<_>>();
    let expected_json = RefactorOutputs {
        candidates: vec![
            CandidateOutput {candidates, is_test: false, crate_name: "hello_world".to_string(), refactoring: "extract-block".to_string()},
            CandidateOutput {candidates: candidates_test, is_test: true, crate_name: "hello_world".to_string(), refactoring: "extract-block".to_string()}
        ],
        refactorings: vec![]
    };
    
    let expected = format!("{}", serde_json::to_string(&expected_json).unwrap());

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
    let map_position = |file: &str, from, to| CandidatePosition { file: file.to_owned(), from, to};
    let map_lib = |candidates, is_test| CandidateOutput {candidates, is_test, crate_name: "lib".to_string(), refactoring: "extract-block".to_string()};
    let map_main = |candidates, is_test| CandidateOutput {candidates, is_test, crate_name: "main".to_string(), refactoring: "extract-block".to_string()};
    let expected_json = RefactorOutputs {
        candidates: vec![
            map_lib(vec![
                map_position("src/lib.rs", 28, 41)], false), 
            map_lib(vec![
                map_position("src/submod.rs", 47, 52),
                map_position("src/lib.rs", 28, 41)
            ], true),
            map_main(vec![
                map_position("src/main.rs", 29, 42)], false),
            map_main(vec![
                map_position("src/submod.rs", 47, 52),
                map_position("src/main.rs", 29, 42)
            ], true)
        ],
        refactorings: vec![]
    };

    let expected = format!("{}", serde_json::to_string(&expected_json).unwrap());

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
        .arg("--selection=0:0")
        .arg("--file=src/lib.rs")
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
