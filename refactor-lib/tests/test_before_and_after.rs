use std::fs::{read_dir, File};
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

/**
 * Test the tests?
 * create new tmp project
 * For each case
 *   set before as main.rs
 *   compile then execute with stdout = s'
 *   set after as main.rs
 *   compile then execute with stdout = s''
 *   assert(s' == s'')
 */

static TEST_CASE_PATH: &str = "../refactor-examples/extract_method";
static _TEST_TMP_PATH: &str = "../tmp";
static _TEST_TMP_PROJECT_NAME: &str = "tmp_project";
static TEST_TMP_PROJECT_DIR: &str = "../tmp/tmp_project";

fn list_tests(dir: &Path) -> std::io::Result<Vec<String>> {
    let mut ret = vec![];
    if dir.is_dir() {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name.ends_with("_after.rs") {
                ret.push(file_name.replace("_after.rs", ""));
            }
        }
    }
    Ok(ret)
}

fn read_test_file(file_path: &str) -> std::io::Result<String> {
    let path = Path::new(TEST_CASE_PATH).join(file_path);
    if !path.is_file() {
        panic!("file didnt exist: {}", path.to_str().unwrap());
    }
    let mut file = File::open(path)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    Ok(file_content)
}

fn compile_and_run() -> std::io::Result<Vec<u8>> {
    let output = Command::new("cargo")
        .args(&["run"])
        .current_dir(TEST_TMP_PROJECT_DIR)
        .output()
        .expect("failed to run");
    if !output.status.success() {
        panic!(String::from_utf8(output.stderr).unwrap());
    }
    Ok(output.stdout)
}

fn do_test_before_and_after() -> std::io::Result<()> {
    let describe = |case, before, after| {
        format!(
            "testcase: {}, \noutput before: `{}`, \noutput after: `{}`",
            case,
            String::from_utf8(before).unwrap(),
            String::from_utf8(after).unwrap()
        )
    };
    create_tmp_project()?;
    for test_case in list_tests(&Path::new(TEST_CASE_PATH))? {
        let before_file_name = format!("{}.rs", &test_case);
        set_main_rs(&read_test_file(&before_file_name)?)?;
        let output_before = compile_and_run()?;

        let after_file_name = format!("{}_after.rs", &test_case);
        set_main_rs(&read_test_file(&after_file_name)?)?;
        let output_after = compile_and_run()?;
        assert_eq!(
            output_before,
            output_after,
            "{}",
            describe(test_case, output_before.clone(), output_after.clone())
        );
        println!("{}", describe(test_case, output_before, output_after));
    }

    Ok(())
}

fn clear_tmp_projects() {}
fn create_tmp_project() -> std::io::Result<()> {
    clear_tmp_projects();

    // Command::new("cargo")
    //     .args(&["new", TEST_TMP_PROJECT_NAME])
    //     .current_dir(TEST_TMP_PATH)
    //     .output()?;
    Ok(())
}

fn set_main_rs(content: &str) -> std::io::Result<()> {
    let path = Path::new(TEST_TMP_PROJECT_DIR).join("src").join("main.rs");
    if !path.is_file() {
        panic!("file didnt exist: {}", path.to_str().unwrap());
    }
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[allow(unused)]
// #[test]
fn test_before_and_after() {
    do_test_before_and_after().unwrap();
}
