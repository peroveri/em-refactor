use refactor_lib_types::{CandidatePosition, RefactoringError, RefactorErrorType};
use serde::Serialize;
use super::{TestResult, TestResults};
use log::info;
use itertools::Itertools;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ReportData {
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
impl ReportData {
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
    pub fn to_report(&self) -> Report {
        Report {
            candidates_found: self.candidates.len(),
            successful: self.result.iter().filter(|e| match e.1 {
                RefactorResult::Success() => true, _ => false }).count(),
            internal_errs: self.result.iter().filter(|e| match e.1 {
                RefactorResult::Err(RefactoringError { kind: RefactorErrorType::Internal, .. }) => true,
                RefactorResult::Err(RefactoringError { kind: RefactorErrorType::RustCError1, .. }) => true,
                _ => false }).count(),
            recompile_errs: self.result.iter().filter(|e| match e.1 {
                RefactorResult::Err(RefactoringError { kind: RefactorErrorType::RustCError2, .. }) => true, _ => false }).count(),
            unit_errs: self.result.iter().filter(|e| match e.1 {
                RefactorResult::UnitErr(..) => true, _ => false }).count(),
            errs_by_micro_refactoring: self.map_errs_by_micro_refactoring()
        }
    }
    fn map_errs_by_micro_refactoring(&self) -> RefaGroup {
        let errs = &self.result.iter().filter_map(|e| match &e.1 {
            RefactorResult::Err(err) => Some(err.clone()),
            _ => None
        }).collect::<Vec<_>>();
        let mut ret = vec![];

        for (refactoring, b) in &errs.into_iter()
            .sorted_by_key(|x| x.at_refactoring.to_string())
            .group_by(|x| x.at_refactoring.to_string()) {
            
            ret.push((refactoring, Self::group_by(&b.collect::<Vec<_>>())));
        }

        ret
    }

    fn group_by(errs: &Vec<&RefactoringError>) -> ErrorsGrouped {
        let mut res = vec![];
            
        let x = &errs.into_iter()
            .sorted_by_key(|k| (k.kind.clone(), k.codes.first().unwrap_or(&"".to_string()).to_string()))
            .group_by(|k| (k.kind.clone(), k.codes.first().unwrap_or(&"".to_string()).to_string()));
        for (y1, y2) in x {
            res.push((y1.0, y1.1, y2.count()));
        }
            
        res
    }
}
type RefaGroup = Vec<(String, ErrorsGrouped)>;
type ErrorsGrouped = Vec<(RefactorErrorType, String, usize)>;
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Report {
    pub candidates_found: usize,
    pub successful: usize,
    pub internal_errs: usize,
    pub recompile_errs: usize,
    pub unit_errs: usize,
    pub errs_by_micro_refactoring: RefaGroup
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
    pub fn from(report: &ReportData) -> Self {
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_to_report() {
        let cand = || CandidatePosition::new("", 0, 0);
        let report = ReportData {
            candidates: vec![],
            refactoring: "foo".to_owned(),
            result: vec![
                (cand(), RefactorResult::Err(RefactoringError{
                    at_refactoring: "intro-closure".to_owned(),
                    codes: vec!["E124".to_owned()],
                    is_error: true,
                    kind: RefactorErrorType::RustCError2,
                    message: "foo".to_owned()
                })),
                (cand(), RefactorResult::Err(RefactoringError{
                    at_refactoring: "extract-block".to_owned(),
                    codes: vec!["E123".to_owned()],
                    is_error: true,
                    kind: RefactorErrorType::RustCError2,
                    message: "foo".to_owned()
                })),
                (cand(), RefactorResult::Success()),
                (cand(), RefactorResult::Err(RefactoringError{
                    at_refactoring: "intro-closure".to_owned(),
                    codes: vec!["E124".to_owned()],
                    is_error: true,
                    kind: RefactorErrorType::RustCError2,
                    message: "foo".to_owned()
                }))
            ],
            test_result: TestResults::new()
        };

        let expected = Report {
            candidates_found: 0,
            errs_by_micro_refactoring: vec![
                ("extract-block".to_owned(), vec![
                    (RefactorErrorType::RustCError2, "E123".to_owned(), 1)
                ]),
                ("intro-closure".to_owned(), vec![
                    (RefactorErrorType::RustCError2, "E124".to_owned(), 2)
                ])
            ],
            internal_errs: 0,
            recompile_errs: 3,
            successful: 1,
            unit_errs: 0
        };

        let actual = report.to_report();

        assert_eq!(expected, actual);
    }
}