use refactor_lib_types::{CandidatePosition, CandidateOutput, RefactoringError};
use serde::Serialize;
use super::{TestResult, TestResults};

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