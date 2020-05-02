use refactor_lib_types::{CandidatePosition, RefactoringError};
use serde::Serialize;
use super::{TestResult, TestResults};
use log::info;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Report {
    pub refactoring: String,
    pub test_result: TestResults,
    pub candidates: Vec<CandidatePosition>,
    pub result: Vec<(CandidatePosition, RefactorResult)>,
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum RefactorResult {
    Err(RefactoringError),
    UnitErr(TestResults),
    Success()
}
impl Report {
    pub fn new(refactoring: String) -> Self {
        Self {
            refactoring,
            candidates: vec![],
            result: vec![],
            test_result: TestResults::new(),
        }
    }
    pub fn add_err(&mut self, candidate: CandidatePosition, err: RefactoringError) {
        info!("Report::add_err {}", self.result.len() + 1);
        info!("Report::add_err candidate: {:?}, err: {:?}", candidate, err);
        self.result.push((candidate, RefactorResult::Err(err)));
    }
    pub fn add_successful(&mut self, candidate: CandidatePosition) {
        info!("Report::add_successful {}", self.result.len() + 1);
        self.result.push((candidate, RefactorResult::Success()));
    }
    pub fn add_unittest_err(&mut self, candidate: CandidatePosition, test_results: TestResults) {
        info!("Report::add_successful {}", self.result.len() + 1);
        self.result.push((candidate, RefactorResult::UnitErr(test_results)));
    }
    pub fn set_candidates(&mut self, candidates: Vec<CandidatePosition>) {
        info!("Report::set_candidates: {}", candidates.len());
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
            errs: report.result.iter().filter(|e| if let RefactorResult::Err(..) = e.1 {true} else {false}).count(),
            success: report.result.iter().filter(|e| if let RefactorResult::Success() = e.1 {true} else {false}).count(),
            test_result: report.test_result.sum.clone(),
            unit_err: report.result.iter().filter(|e| if let RefactorResult::UnitErr(..) = e.1 {true} else {false}).count(),
        }
    }
}