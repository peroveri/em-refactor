use crate::output_types::{FileStringReplacement, RefactoringError, RefactorOutput};
use crate::refactoring_invocation::{arg_value, InternalErrorCodes, RefactoringErrorInternal};

pub fn from_success(rustc_args: &[String], replacements: Vec<FileStringReplacement>) -> RefactorOutput {
    RefactorOutput {
        crate_name: arg_value(rustc_args, "--crate-name", |_| true).unwrap().to_owned(),
        is_test: rustc_args.contains(&"--test".to_owned()),
        replacements: replacements,
        errors: vec![]
    }
}
    
pub fn from_error(rustc_args: &[String], error: RefactoringErrorInternal) -> RefactorOutput {
    RefactorOutput {
        crate_name: arg_value(rustc_args, "--crate-name", |_| true).unwrap_or("").to_owned(),
        is_test: rustc_args.contains(&"--test".to_owned()),
        replacements: vec![],
        errors: vec![RefactoringError {
            message: error.message,
            is_error: error.code != InternalErrorCodes::FileNotFound
        }]
    }
}