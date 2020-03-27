use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorOutput {
    pub crate_name: String,
    pub is_test: bool,
    pub replacements: Vec<FileStringReplacement>,
    pub errors: Vec<RefactoringError>
}
