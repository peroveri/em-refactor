use super::{CmdRunner, Report, ShortReport};
use refactor_lib_types::CandidatePosition;
use std::path::PathBuf;

/// # Algo (Extract Method)
/// Given Project / Crate
/// C  <- Candidates (Extract block)
/// T' <- Run unit tests
/// 
/// for each C' in C:
///     if ok(diff) = refactor(C'):
///         Apply diff to fs
///         T'' <- Run unit tests
///         Undo diff to fs
///     else err() = ..:
///         Log err, cand
struct ExperimentsRunner {
    refactorings: Vec<String>,
    cmd_runner: CmdRunner,
    report: Report
}

impl ExperimentsRunner {
    pub fn new(refactorings: Vec<String>, cmd_runner: CmdRunner) -> Self {
        Self {
            report: Report::new(&refactorings[0]),
            refactorings,
            cmd_runner
        }
    }

    fn run_exp_on_project(&mut self) -> std::io::Result<()> {
        if self.cmd_runner.has_repo_changes()? {
            println!("repo has changes");
            return Ok(());
        }
        self.report.test_result = self.cmd_runner.run_unit_tests()?;
        self.report.candidates = self.cmd_runner.query_candidates(&self.refactorings[0])?.candidates;

        for candidates_crate in self.report.candidates.clone().iter().filter(|c| !c.is_test) {
            for candidate in &candidates_crate.candidates {
                self.run_candidate_refactoring(candidate, &candidates_crate.refactoring)?;
            }
        }
        Ok(())
    }

    fn run_candidate_refactoring(&mut self, candidate: &CandidatePosition, refactoring: &str) -> std::io::Result<()> {
        let changes = self.cmd_runner.refactor(candidate, refactoring)?;
        let err = changes.refactorings.iter()
            .find_map(|r| r.errors.iter().find(|e| e.is_error));
        
        if let Some(err) = err {
            self.report.errs.push((candidate.clone(), err.clone()));
        } else {
            self.cmd_runner.apply_changes(changes)?;
            let next_test_result = self.cmd_runner.run_unit_tests()?;
            if !next_test_result.eq(&self.report.test_result) {
                self.report.unit_err.push((candidate.clone(), next_test_result));
            } else {
                self.report.success.push(candidate.clone());
            }
            self.cmd_runner.reset_repo()?;
        }
        
        Ok(())
    }
}

pub fn run_all_exp(refactoring: &str, crate_path: &str) -> std::io::Result<()> {
    // std::env::set_current_dir(crate_path).unwrap();
    let mut tool_path = std::env::current_exe()
        .expect("current executable path invalid")
        .with_file_name("cargo-my-refactor");
    if cfg!(windows) {
        tool_path.set_extension("exe");
    }
    let refactorings = 
        if refactoring == "extract-method" {
            vec!["extract-block".to_owned()]
        } else {
            panic!()
        };
    let cmd_runner = CmdRunner::new_default_tmp_dir(&PathBuf::from(crate_path), tool_path);
    let mut experiments_runner = ExperimentsRunner::new(refactorings, cmd_runner);
    experiments_runner.run_exp_on_project()?;
    println!("{}", serde_json::to_string(&experiments_runner.report).unwrap());
    println!("{}", serde_json::to_string(&ShortReport::from(&experiments_runner.report)).unwrap());
    Ok(())
}
