use serde::{Serialize, Deserialize};
use crate::refactoring_invocation::{arg_value, InternalErrorCodes, RefactoringErrorInternal};

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

impl RefactorOutput {
    pub fn from_success(rustc_args: &[String], replacements: Vec<FileStringReplacement>) -> Self {
        Self {
            crate_name: arg_value(rustc_args, "--crate-name", |_| true).unwrap().to_owned(),
            is_test: rustc_args.contains(&"--test".to_owned()),
            replacements: replacements,
            errors: vec![]
        }
    }
    
    pub fn from_error(rustc_args: &[String], error: RefactoringErrorInternal) -> Self {
        Self {
            crate_name: arg_value(rustc_args, "--crate-name", |_| true).unwrap_or("").to_owned(),
            is_test: rustc_args.contains(&"--test".to_owned()),
            replacements: vec![],
            errors: vec![RefactoringError {
                message: error.message,
                is_error: error.code != InternalErrorCodes::FileNotFound
            }]
        }
    }
}
