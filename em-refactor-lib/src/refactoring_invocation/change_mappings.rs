use em_refactor_lib_types::{FileStringReplacement, RefactoringError, RefactorOutput, RefactorOutputs};
use crate::refactoring_invocation::{arg_value, RefactoringErrorInternal};

pub fn from_success(rustc_args: &[String], replacements: Vec<FileStringReplacement>) -> RefactorOutputs {
    RefactorOutputs::from_refactorings(vec![
        RefactorOutput {
            crate_name: arg_value(rustc_args, "--crate-name", |_| true).unwrap().to_owned(),
            is_test: rustc_args.contains(&"--test".to_owned()),
            replacements: replacements,
            errors: vec![]
        }
    ])
}
    
pub fn from_error(rustc_args: &[String], error: RefactoringErrorInternal, refactoring: &str) -> RefactorOutputs {
    RefactorOutputs::from_refactorings(vec![
        RefactorOutput {
            crate_name: arg_value(rustc_args, "--crate-name", |_| true).unwrap_or("").to_owned(),
            is_test: rustc_args.contains(&"--test".to_owned()),
            replacements: vec![],
            errors: vec![RefactoringError {
                message: error.message,
                is_error: error.is_error,
                kind: error.error_type,
                codes: error.external_codes,
                at_refactoring: refactoring.to_string()
            }]
        }
    ])
}