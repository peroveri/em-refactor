use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use refactor_lib_types::{CandidatePosition, RefactorOutputs2};
use super::TestResults;
use itertools::Itertools;
use log::info;

pub struct CmdRunner {
    crate_path: PathBuf,
    tool_path: PathBuf,
    _tmp_dir: TempDir
}

impl CmdRunner {
    pub fn new(crate_path: &PathBuf, tool_path: PathBuf, _tmp_dir: TempDir) -> Self {
        Self {
            crate_path: crate_path.clone(),
            tool_path: tool_path.clone(),
            _tmp_dir
        }
    }
    pub fn new_default_tmp_dir(crate_path: &PathBuf, tool_path: PathBuf) -> Self {
        Self::new(crate_path, tool_path, create_tmp_dir())
    }

    pub fn has_repo_changes(&self) -> std::io::Result<bool> {
        Ok(self.has_repo_staged_changes(true)? || self.has_repo_staged_changes(false)?)
    }
    pub fn has_repo_staged_changes(&self, staged: bool) -> std::io::Result<bool> {
        let mut args = vec!["--quiet", "--exit-code"];
        if staged {
            args.push("--cached");
        }
        let out = Command::new("git")
            .arg("diff")
            .args(args)
            .current_dir(&self.crate_path)
            .output()?;
        Ok(!out.status.success())
    }
    pub fn reset_repo(&self) -> std::io::Result<()> {
        let out = Command::new("git")
            .arg("reset")
            .arg("--hard")
            .current_dir(&self.crate_path)
            .output()?;
        
        assert!(out.status.success());
        Ok(())
    }
    pub fn query_candidates(&self, refactoring: &str) -> std::io::Result<RefactorOutputs2> {
        let output = 
            Command::new(&self.tool_path)
                .current_dir(&self.crate_path)
                .arg("--target-dir=target/refactorings")
                // .arg("--target-dir")
                // .arg(self.tmp_dir.path())
                .arg("candidates")
                .arg(refactoring)
                .output()?;
        
        assert_eq!(output.status.code(), Some(0), "stdout: {}\nstderr:{}", std::str::from_utf8(output.stdout.as_slice()).unwrap(), std::str::from_utf8(output.stderr.as_slice()).unwrap());
        
        let s = std::str::from_utf8(output.stdout.as_slice()).unwrap();

        Ok(serde_json::from_str(s).unwrap())
    }
    pub fn refactor(&self, candidate: &CandidatePosition, refactoring: &str) -> std::io::Result<RefactorOutputs2> {
        let output = Command::new(&self.tool_path)
            .current_dir(&self.crate_path)
            .arg("--target-dir=target/refactorings")
            .arg("refactor")
            .arg(refactoring)
            .arg(&candidate.file)
            .arg(format!("{}:{}", candidate.from, candidate.to))
            .output().unwrap();

        assert!(output.status.success(), "stdout: {}\nstderr:{}\ncandidate: {:?}", std::str::from_utf8(output.stdout.as_slice()).unwrap(), std::str::from_utf8(output.stderr.as_slice()).unwrap(), candidate);
        
    
        let stdout = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        Ok(serde_json::from_str(stdout).unwrap())
    }
    pub fn apply_changes(&self, output: RefactorOutputs2) -> std::io::Result<()> {
        
        for changes in &output.changes {
            for (file_path, changes) in &changes.into_iter()
                .sorted_by_key(|a| a.file_name.clone())
                .group_by(|a| a.file_name.clone()) {
                let changes = changes.collect::<Vec<_>>();

                let path: PathBuf = [&self.crate_path, &PathBuf::from(&file_path)].iter().collect();
                
                let mut file = File::open(&path)?;
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                
                for change in &changes {
                    let s1 = &content[..(change.byte_start) as usize];
                    let s2 = &content[(change.byte_end) as usize..];
                    content = format!("{}{}{}", s1, change.replacement, s2);
                }
                info!("apply_changes: {:?}", &path);
                let mut file = File::create(&path)?;
                file.write_all(content.as_bytes())?;
            }
        }

        Ok(())
    }
    pub fn run_unit_tests(&self) -> std::io::Result<TestResults> {
    
        let out = Command::new("cargo")
            .arg("test")
            .arg("--no-fail-fast")
            .arg("--all-targets")
            .current_dir(&self.crate_path)
            .output()?;
    
        let s1 = std::str::from_utf8(out.stdout.as_slice()).unwrap();
    
        let result = TestResults::from(s1)?;
    
        Ok(result)
    }
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