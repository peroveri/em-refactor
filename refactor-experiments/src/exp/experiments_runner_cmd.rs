use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use refactor_lib_types::{CandidatePosition, RefactorOutputs};
use super::TestResults;
use itertools::Itertools;

pub struct CmdRunner {
    crate_path: PathBuf,
    tool_path: PathBuf,
    tmp_dir: TempDir
}

impl CmdRunner {
    pub fn new(crate_path: &PathBuf, tool_path: PathBuf, tmp_dir: TempDir) -> Self {
        Self {
            crate_path: crate_path.clone(),
            tool_path: tool_path.clone(),
            tmp_dir
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
    pub fn query_candidates(&self, refactoring: &str) -> std::io::Result<RefactorOutputs> {
        let output = 
            Command::new(&self.tool_path)
                .arg("--workspace-root")
                .arg(&self.crate_path)
                .arg("--target-dir")
                .arg(self.tmp_dir.path())
                .arg("candidates")
                .arg(refactoring)
                .output()?;
        
        assert_eq!(output.status.code(), Some(0));
        
        let s = std::str::from_utf8(output.stdout.as_slice()).unwrap();

        Ok(serde_json::from_str(s).unwrap())
    }
    pub fn refactor(&self, candidate: &CandidatePosition, refactoring: &str) -> std::io::Result<RefactorOutputs> {
        let output = Command::new(&self.tool_path)
            .arg("--workspace-root")
            .arg(&self.crate_path)
            .arg("--target-dir")
            .arg(self.tmp_dir.path())
            .arg("refactor")
            .arg(refactoring)
            .arg(&candidate.file)
            .arg(format!("{}:{}", candidate.from, candidate.to))
            .arg("--output-replacements-as-json")
            .output().unwrap();

        assert!(output.status.success());
    
        let stdout = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        Ok(serde_json::from_str(stdout).unwrap())
    }
    pub fn apply_changes(&self, changes: RefactorOutputs) -> std::io::Result<()> {
        
        let replacements = changes.refactorings.iter()
            .flat_map(|r| &r.replacements).collect::<Vec<_>>();
        let r = replacements.iter().unique().collect::<Vec<_>>();

        for (file_path, changes) in &r.into_iter().group_by(|a| a.file_name.clone()) {

            let mut changes = changes.collect::<Vec<_>>();
            changes.sort_by_key(|c| c.byte_start);
            changes.reverse();

            let path: PathBuf = [&self.crate_path, &PathBuf::from(&file_path)].iter().collect();
            
            let mut file = File::open(&path)?;
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            
            for change in &changes {
                let s1 = &content[..(change.byte_start) as usize];
                let s2 = &content[(change.byte_end) as usize..];
                content = format!("{}{}{}", s1, change.replacement, s2);
            }
            println!("writing to: {:?}", &path);
            let mut file = File::create(&path)?;
            file.write_all(content.as_bytes())?;
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