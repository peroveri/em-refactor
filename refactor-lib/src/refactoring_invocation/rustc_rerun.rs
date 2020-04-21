use refactor_lib_types::FileStringReplacement;
use crate::refactoring_invocation::{RefactoringErrorInternal, InMemoryFileLoader};

struct DefaultCallbacks;
impl rustc_driver::Callbacks for DefaultCallbacks {}
pub fn rustc_rerun(changes: &Vec<FileStringReplacement>, rustc_args: &[String]) -> Result<(), RefactoringErrorInternal> {
    let mut default = DefaultCallbacks;

    let mut file_loader = Box::new(InMemoryFileLoader::new(
        rustc_span::source_map::RealFileLoader,
    ));
    file_loader.add_changes(changes.clone());

    let emitter = Box::new(Vec::new());
    let err =
        rustc_driver::run_compiler(&rustc_args, &mut default, Some(file_loader), Some(emitter));
    // let err = rustc_driver::catch_fatal_errors(|| {
    //     let err = rustc_driver::run_compiler(&rustc_args, &mut default, Some(file_loader), Some(emitter));
    //     if let Err(err) = err {
    //         return Err(err);
    //     }
    //     err
    // });

    if err.is_err() {
        return Err(RefactoringErrorInternal::int("The refactoring broke the code"));
    }
    return Ok(());
    // TODO: output message / status that the code was broken after refactoring
}