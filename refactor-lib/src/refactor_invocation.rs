use super::RefactorStatusCodes;
use super::my_refactor_callbacks;
use super::change::FileReplaceContent;
use super::change::Change;
use super::refactor_definition::RefactoringError;

pub fn run_refactoring(rustc_args: &[String], refactor_def: super::refactor_definition::RefactorDefinition, refactor_args: &[String]) -> Result<Option<(String, Vec<FileReplaceContent>, Result<Vec<Change>, RefactoringError>)>, i32> {

    let mut my_refactor = my_refactor_callbacks::MyRefactorCallbacks::from_arg(refactor_def);

    let callbacks: &mut (dyn rustc_driver::Callbacks + Send) = &mut my_refactor;

    std::env::set_var("RUST_BACKTRACE", "1");

    let emitter = Box::new(Vec::new());
    // TODO: looks like the errors are not caught here?
    // Should set own errors on the Callbacks struct
    let err = rustc_driver::run_compiler(&rustc_args, callbacks, None, Some(emitter));
    // let err = rustc_driver::catch_fatal_errors(|| {
    //     rustc_driver::run_compiler(&rustc_args, callbacks, None, Some(emitter))
    // });
    if err.is_err() {
        if let Some(msg) = my_refactor.content {
            eprintln!("{}", msg);
        } else {
            eprintln!("failed during refactoring");
        }
        return Err(RefactorStatusCodes::InputDoesNotCompile as i32);
    }
    let content = my_refactor.content.clone().unwrap_or_else(|| "".to_owned());
    let replacements = my_refactor.file_replace_content.clone();

    if let Err(err) = my_refactor.result {
        if err.code == crate::refactor_definition::InternalErrorCodes::FileNotFound &&
        refactor_args.contains(&"--ignore-missing-file".to_owned()) {
            return Ok(None);
        }
        eprintln!("{}", err.message);
        return Err(RefactorStatusCodes::InternalRefactoringError as i32);
    }

    return Ok(Some((content, replacements, my_refactor.result)));
}