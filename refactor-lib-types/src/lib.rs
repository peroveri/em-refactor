use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileStringReplacement {
    pub byte_end: u32,
    pub byte_start: u32,
    pub char_end: usize,
    pub char_start: usize,
    pub file_name: String,
    pub line_end: usize,
    pub line_start: usize,
    pub replacement: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactoringError {
    // pub byte_end: u32,
    // pub byte_start: u32,
    // pub char_end: usize,
    // pub char_start: usize,
    // pub file_name: String,
    // pub line_end: usize,
    // pub line_start: usize,
    pub is_error: bool,
    pub message: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactorOutput {
    pub crate_name: String,
    pub is_test: bool,
    pub replacements: Vec<FileStringReplacement>,
    pub errors: Vec<RefactoringError>
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactorOutputs {
    pub candidates: Vec<CandidateOutput>,
    pub refactorings: Vec<RefactorOutput>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateOutput {
    pub candidates: Vec<CandidatePosition>,
    pub crate_name: String,
    pub is_test: bool,
    pub refactoring: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidatePosition {
    pub file: String,
    pub from: u32,
    pub to: u32
}

impl RefactorOutputs {
    #[allow(unused)]
    pub fn sort(&mut self) {
        self.candidates.sort_by_key(|a| (a.crate_name.clone(), a.is_test));
        self.refactorings.sort_by_key(|a| (a.crate_name.clone(), a.is_test))
    }
    #[allow(unused)]
    pub fn extend(&mut self, other: RefactorOutputs) {
        self.candidates.extend(other.candidates);
        self.refactorings.extend(other.refactorings);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactorArgs {
    pub refactoring: Option<String>,
    pub selection: Option<String>,
    pub query_candidates: Option<String>,
    pub file: Option<String>,
    pub usafe: bool,
    pub single_file: bool,
    pub output_replacements_as_json: bool,
}