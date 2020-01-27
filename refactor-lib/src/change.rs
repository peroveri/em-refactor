use serde::{Serialize, Deserialize};
use super::arg_mappings::arg_value;

/// 
/// Represents a file change applied by the refactorings
/// 
#[derive(Clone)]
pub struct Change {
    pub end: u32,
    pub file_name: String,
    pub file_start_pos: u32,
    /// Indexed relative to this file
    /// Indexed relative to this file
    pub replacement: String,
    pub start: u32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReplaceContent {
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
    // pub root_path: String,
    pub is_test: bool,
    pub replacements: Vec<FileReplaceContent>,
    pub errors: Vec<RefactoringError>
}

pub fn map_success_to_output(rustc_args: &[String], replacements: Vec<FileReplaceContent>) -> RefactorOutput {
    RefactorOutput {
        crate_name: arg_value(rustc_args, "--crate-name", |_| true).unwrap().to_owned(),
        is_test: rustc_args.contains(&"--test".to_owned()),
        replacements: replacements,
        errors: vec![]
        // root_path: "".to_owned()
    }
}

pub fn map_fail_to_output(rustc_args: &[String], error: crate::refactor_definition::RefactoringError) -> RefactorOutput {
    RefactorOutput {
        crate_name: arg_value(rustc_args, "--crate-name", |_| true).unwrap_or("").to_owned(),
        is_test: rustc_args.contains(&"--test".to_owned()),
        replacements: vec![],
        errors: vec![RefactoringError {
            message: error.message,
            is_error: error.code != crate::refactor_definition::InternalErrorCodes::FileNotFound
        }]
        // root_path: "".to_owned()
    }
}