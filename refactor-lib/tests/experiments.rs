use assert_cmd::prelude::*;
use exp::TestResults;
use std::io::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use serde::{Serialize, Deserialize};

const WORK_DIR: &str = "./tests/exp/work_dir";

mod exp;

fn repo_name(url: &str) -> Option<String> {
    Some(url[1+url.rfind("/")?..].to_string())
}

fn debug(s: &str) -> std::io::Result<()> {
    let mut f =  std::fs::OpenOptions::new()
        .append(true)
        .open("./tests/exp/work_dir/debug.txt")?;
    f.write_all(s.as_bytes())?;
    Ok(())
}
fn init() -> std::io::Result<()> {
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
#[test]
#[ignore]
fn experiments_run_tests() -> std::io::Result<()> {
    init()?;
    debug("settings:\n")?;
    let settings = exp::read_settings()?;
    debug(&format!("settings: {}\n", settings.projects.len()))?;

    for project in settings.projects {
        if project.skip.unwrap_or(false) {
            debug(&format!("skipping: {}\n", project.git_repo))?;
            continue;
        }
        let repo_name = repo_name(&project.git_repo).unwrap();

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
            .arg(format!("{}", project.git_repo))
            .status()?;
        }
        let mut p = std::path::PathBuf::new();
        p.push(WORK_DIR);
        p.push(&repo_name);
        if let Some(d) = project.subdir {
            p.push(d);
        }
        
        let absp = std::fs::canonicalize(p)?;
        if !absp.is_dir() {
            panic!("{} is not a dir", absp.display());
        }

        let out = Command::new("cargo")
            .arg("test")
            .arg("--no-fail-fast")
            .arg("--")
            .arg("--test-threads=1")
            .current_dir(&absp)
            .output()?;

        // let res = out.stdout;

        let s1 = std::str::from_utf8(out.stdout.as_slice()).unwrap();
        // let s2 = std::str::from_utf8(out.stderr.as_slice()).unwrap();

        let result = TestResults::from(s1)?.sum();

        let p1: std::path::PathBuf = ["./tests/exp/work_dir", &format!("{}.json", repo_name)].iter().collect();
        let mut f = std::fs::File::create(p1)?;
        f.write_all(serde_json::to_string(&result)?.as_bytes())?;

        let output = 
            Command::cargo_bin("cargo-my-refactor")
                .unwrap()
                .arg("--query-candidates=box-named-field")
                .arg("--")
                .arg(format!(
                    "--target-dir={}",
                    create_tmp_dir().path().to_str().unwrap()
                ))
                .current_dir(&absp)
                .output().unwrap();

        assert_eq!(output.status.code(), Some(0));
        
        let s = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        debug(s)?;
        for line in s.split("\n") {
            if line.trim().is_empty() {
                continue;
            }
            let v: CandidateOutput = serde_json::from_str(line).unwrap();
            
            for x in v.candidates {
                run_refactoring(&v.refactoring, x.from, x.to, &x.file, &absp)?;
            }

            break; // just take the first result for now, should combine candidates through different crates
        }
    }


    Ok(())
}

fn run_refactoring(refactoring: &str, from: u32, to: u32, file: &str, dir: &std::path::PathBuf) -> std::io::Result<()> {
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
    debug(&format!("stdout: {}\n", stderr))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateOutput {
    pub candidates: Vec<CandidatePosition>,
    pub crate_name: String,
    pub refactoring: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePosition {
    pub file: String,
    pub from: u32,
    pub to: u32
}