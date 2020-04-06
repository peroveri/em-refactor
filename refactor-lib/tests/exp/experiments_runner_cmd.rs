use assert_cmd::prelude::*;
use std::io::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use my_refactor_lib::RefactorOutputs;
use super::TestResults;

const WORK_DIR: &str = "./tests/exp/work_dir";

pub fn run_refactoring(refactoring: &str, from: u32, to: u32, file: &str, dir: &std::path::PathBuf) -> std::io::Result<()> {
    debug(&format!("trying: {}\n", refactoring))?;
    let output = Command::cargo_bin("cargo-my-refactor")
        .unwrap()
        .arg("--output-replacements-as-json")
        .arg(format!("--refactoring={}", refactoring))
        .arg(format!("--selection={}:{}", from, to))
        .arg(format!("--file={}", file))
        .arg("--")
        .arg(format!(
            "--target-dir={}",
            create_tmp_dir().path().to_str().unwrap()
        ))
        .current_dir(dir)
        .output().unwrap();

    let stdout = std::str::from_utf8(output.stdout.as_slice()).unwrap();
    let stderr = std::str::from_utf8(output.stderr.as_slice()).unwrap();
    debug(&format!("stdout: {}\n", stdout))?;
    debug(&format!("stderr: {}\n", stderr))?;

    Ok(())
}
pub fn get_absp(repo_name: &str, subdir: &Option<String>) -> std::io::Result<std::path::PathBuf> {

    let mut p = std::path::PathBuf::new();
    p.push(WORK_DIR);
    p.push(&repo_name);
    if let Some(d) = subdir {
        p.push(d);
    }
    
    let absp = std::fs::canonicalize(p)?;
    if !absp.is_dir() {
        panic!("{} is not a dir", absp.display());
    }
    Ok(absp)
}
pub fn run_unit_tests(absp: &std::path::PathBuf, repo_name: &str) -> std::io::Result<()> {

    let out = Command::new("cargo")
        .arg("test")
        .arg("--no-fail-fast")
        .arg("--")
        .arg("--test-threads=1")
        .current_dir(absp)
        .output()?;

    // let res = out.stdout;

    let s1 = std::str::from_utf8(out.stdout.as_slice()).unwrap();
    // let s2 = std::str::from_utf8(out.stderr.as_slice()).unwrap();

    let result = TestResults::from(s1)?.sum();

    let p1: std::path::PathBuf = ["./tests/exp/work_dir", &format!("{}.json", repo_name)].iter().collect();
    let mut f = std::fs::File::create(p1)?;
    f.write_all(serde_json::to_string(&result)?.as_bytes())?;
    Ok(())
}
pub fn write_result(content: &str, name: &str) -> std::io::Result<()> {
    let p1: std::path::PathBuf = ["./tests/exp/work_dir", &format!("{}.json", name)].iter().collect();
    let mut f = std::fs::File::create(p1)?;
    f.write_all(content.as_bytes())?;
    Ok(())
}
pub fn query_candidates(absp: &std::path::PathBuf, refactoring: &str) -> std::io::Result<String> {

    let output = 
        Command::cargo_bin("cargo-my-refactor")
            .unwrap()
            .arg(format!("--query-candidates={}", refactoring))
            .arg("--")
            .arg(format!(
                "--target-dir={}",
                create_tmp_dir().path().to_str().unwrap()
            ))
            .current_dir(absp)
            .output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    
    let s = std::str::from_utf8(output.stdout.as_slice()).unwrap();
    debug(s)?;
    Ok(s.to_string())
}
pub fn map_candidates(s: &str) -> RefactorOutputs {
    serde_json::from_str(s).unwrap()
}
pub fn clone_project(repo_name: &str, git_repo: &str) -> std::io::Result<()> {
    let w: std::path::PathBuf = [WORK_DIR].iter().collect();
    if !w.exists() {
        panic!("path doesnt exist: {}", WORK_DIR);
    }
    let p: std::path::PathBuf = [WORK_DIR, &repo_name].iter().collect();
    if !p.exists() {
        let abs = std::fs::canonicalize(w)?;
        Command::new("git")
        .current_dir(abs)
        .arg("clone")
        .arg(format!("{}", git_repo))
        .status()?;
    }
    Ok(())
}

pub fn repo_name(url: &str) -> Option<String> {
    Some(url[1+url.rfind("/")?..].to_string())
}

pub fn debug(s: &str) -> std::io::Result<()> {
    let mut f =  std::fs::OpenOptions::new()
        .append(true)
        .open("./tests/exp/work_dir/debug.txt")?;
    f.write_all(s.as_bytes())?;
    Ok(())
}
pub fn init() -> std::io::Result<()> {
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("./tests/exp/work_dir/debug.txt")?;
    // f.flush()?;
    Ok(())
}

pub fn create_tmp_dir() -> TempDir {
    let tmp_dir = TempDir::new().unwrap();
    let tmp_dir_path = tmp_dir.path();
    assert!(
        tmp_dir_path.is_dir(),
        "failed to create tmp dir: {}",
        tmp_dir_path.to_str().unwrap_or("")
    );
    tmp_dir
}