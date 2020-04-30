use super::{CmdRunner, Report, ShortReport, Stopwatch};
use refactor_lib_types::CandidatePosition;
use std::path::PathBuf;
use log::{error, info};

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
            error!("repo has changes");
            return Ok(());
        }
        let refactoring = self.refactorings[0].clone();
        self.report.set_test_result(self.cmd_runner.run_unit_tests()?);
        self.report.set_candidates(self.cmd_runner.query_candidates(&refactoring)?.candidates);

        for candidate in self.report.candidates.clone() {
            self.run_candidate_refactoring(candidate, &refactoring)?;
        }
        Ok(())
    }

    fn run_candidate_refactoring(&mut self, candidate: CandidatePosition, refactoring: &str) -> std::io::Result<()> {
        let mut stopwatch = Stopwatch::start("run_candidate_refactoring".to_owned());
        let changes = self.cmd_runner.refactor(&candidate, refactoring)?;
        
        if let Some(err) = changes.errors.first() {
            self.report.add_err(candidate.clone(), err.clone());
        } else {
            self.cmd_runner.apply_changes(changes)?;
            let next_test_result = self.cmd_runner.run_unit_tests()?;
            if !next_test_result.eq(&self.report.test_result) {
                self.report.add_unittest_err(candidate.clone(), next_test_result);
            } else {
                self.report.add_successful(candidate.clone());
            }
            self.cmd_runner.reset_repo()?;
        }
        stopwatch.add("apply_change".to_owned());
        info!("{}", stopwatch.report().unwrap());
        
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
            panic!("Unexpected refactoring: {}", refactoring)
        };
    let cmd_runner = CmdRunner::new_default_tmp_dir(&PathBuf::from(crate_path), tool_path);
    let mut experiments_runner = ExperimentsRunner::new(refactorings, cmd_runner);
    experiments_runner.run_exp_on_project()?;
    println!("{}", serde_json::to_string(&experiments_runner.report).unwrap());
    println!("{}", serde_json::to_string(&ShortReport::from(&experiments_runner.report)).unwrap());
    Ok(())
}
