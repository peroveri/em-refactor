use std::process::Command;
use exp::TestResults;
use std::io::prelude::*;

const WORK_DIR: &str = "./tests/exp/work_dir";

mod exp;

fn repo_name(url: &str) -> Option<String> {
    Some(url[1+url.rfind("/")?..].to_string())
}

#[test]
#[ignore]
fn experiments_run_tests() -> std::io::Result<()> {
    let settings = exp::read_settings()?;

    for project in settings.projects {
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
            .current_dir(absp)
            .output()?;

        // let res = out.stdout;

        let s1 = std::str::from_utf8(out.stdout.as_slice()).unwrap();
        // let s2 = std::str::from_utf8(out.stderr.as_slice()).unwrap();

        let result = TestResults::from(s1)?.sum();

        let p1: std::path::PathBuf = ["./tests/exp/work_dir", &format!("{}.json", repo_name)].iter().collect();
        let mut f = std::fs::File::create(p1)?;
        f.write_all(serde_json::to_string(&result)?.as_bytes())?;
    }


    Ok(())
}
