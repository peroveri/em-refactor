use refactor_lib_types::{CandidatePosition, CandidateOutput, RefactoringError};
use serde::Serialize;
use super::{TestResult, TestResults};
use log::info;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Report {
    pub refactoring: String,
    pub test_result: TestResults,
    pub candidates: Vec<CandidateOutput>,
    pub errs: Vec<(CandidatePosition, RefactoringError)>,
    pub unit_err: Vec<(CandidatePosition, TestResults)>,
    pub success: Vec<CandidatePosition>
}
impl Report {
    pub fn new(refactoring: &str) -> Self {
        Self {
            refactoring: refactoring.to_string(),
            candidates: vec![],
            errs: vec![],
            success: vec![],
            test_result: TestResults::new(),
            unit_err: vec![]
        }
    }
    pub fn add_err(&mut self, candidate: CandidatePosition, err: RefactoringError) {
        info!("Report::add_err {}", self.errs.len() + 1);
        info!("Report::add_err candidate: {:?}, err: {:?}", candidate, err);
        self.errs.push((candidate, err));
    }
    pub fn add_successful(&mut self, candidate: CandidatePosition) {
        info!("Report::add_successful {}", self.success.len() + 1);
        self.success.push(candidate);
    }
    pub fn set_candidates(&mut self, candidates: Vec<CandidateOutput>) {
        info!("Report::set_candidates: {}", candidates.iter().map(|c| c.candidates.len().to_string()).collect::<Vec<_>>().join(", "));
        self.candidates = candidates;
    }
    pub fn set_test_result(&mut self, test_result: TestResults) {
        info!("Report::set_test_result: {}", test_result.to_single_line());
        self.test_result = test_result;
    }
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ShortReport {
    pub refactoring: String,
    pub test_result: TestResult,
    pub candidates: usize,
    pub errs: usize,
    pub unit_err: usize,
    pub success: usize
}

impl ShortReport {
    pub fn from(report: &Report) -> Self {
        Self {
            refactoring: report.refactoring.clone(),
            candidates: report.candidates.len(),
            errs: report.errs.len(),
            success: report.success.len(),
            test_result: report.test_result.sum.clone(),
            unit_err: report.unit_err.len()
        }
    }
}