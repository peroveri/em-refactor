use super::{CmdRunner, ExperimentsOutput, ReportData, Stopwatch};
use em_refactor_lib_types::CandidatePosition;
use std::path::PathBuf;
use log::{error, info};
use std::fs::File;
use std::io::prelude::*;

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
    refactoring: String,
    cmd_runner: CmdRunner,
    report: ReportData
}

impl ExperimentsRunner {
    pub fn new(refactoring: String, cmd_runner: CmdRunner) -> Self {
        Self {
            report: ReportData::new(refactoring.clone()),
            refactoring,
            cmd_runner
        }
    }

    fn run_exp_on_project(&mut self) -> std::io::Result<()> {
        if self.cmd_runner.has_repo_changes()? {
            error!("repo has changes");
            return Ok(());
        }
        self.report.set_test_result(self.cmd_runner.run_unit_tests()?);
        self.report.set_candidates(self.cmd_runner.query_candidates(&self.refactoring)?.candidates);
        for candidate in self.report.candidates.clone() {
            self.run_candidate_refactoring(candidate)?;
        }
        Ok(())
    }

    fn run_candidate_refactoring(&mut self, candidate: CandidatePosition) -> std::io::Result<()> {
        let mut stopwatch = Stopwatch::start("run_candidate_refactoring".to_owned());
        let changes = self.cmd_runner.refactor(&candidate, &self.refactoring)?;
        stopwatch.add("refactor");
        
        if let Some(err) = changes.errors.first() {
            self.report.add_err(candidate.clone(), err.clone());
            stopwatch.add("add_err");
        } else {
            self.cmd_runner.apply_changes(changes)?;
            let next_test_result = self.cmd_runner.run_unit_tests()?;
            if !next_test_result.eq(&self.report.test_result) {
                self.report.add_unittest_err(candidate.clone(), next_test_result);
            } else {
                self.report.add_successful(candidate.clone());
            }
            self.cmd_runner.reset_repo()?;
            stopwatch.add("reset_repo");
        }
        info!("{}", stopwatch.report().unwrap());
        
        Ok(())
    }
}

pub fn run_all_exp(options: ExperimentOptions) -> std::io::Result<()> {
    let mut tool_path = std::env::current_exe()
        .expect("current executable path invalid")
        .with_file_name("cargo-em-refactor");
    if cfg!(windows) {
        tool_path.set_extension("exe");
    }
    let cmd_runner = CmdRunner::new_default_tmp_dir(&PathBuf::from(&options.workspace_root), tool_path);
    let mut experiments_runner = ExperimentsRunner::new(options.refactoring.to_string(), cmd_runner);
    experiments_runner.run_exp_on_project()?;

    let output = serde_json::to_string(&ExperimentsOutput::create(&experiments_runner.report)).unwrap();
    let mut file = File::create(format!("{}.report.json", options.get_file_prefix()))?;
    file.write_all(output.as_bytes())?;
    Ok(())
}
pub struct ExperimentOptions {
    pub refactoring: String,
    pub started_at: String,
    pub workspace_root: String,
    pub log_to_file: bool
}

impl ExperimentOptions {
    pub fn get_file_prefix(&self) -> String {
         format!("{}_{}", self.refactoring, self.started_at)
    }
}