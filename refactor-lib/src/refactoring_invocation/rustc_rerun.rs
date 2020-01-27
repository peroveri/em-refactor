use super::file_loader::InMemoryFileLoader;
use crate::change::FileReplaceContent;
use crate::RefactorStatusCodes;

pub fn should_run_rustc_again(refactor_args: &[String]) -> bool {
    return !refactor_args.contains(&"--unsafe".to_owned());
}
pub fn rustc_rerun(changes: &Vec<FileReplaceContent>, rustc_args: &[String]) -> Result<(), i32> {
    let mut default = rustc_driver::DefaultCallbacks;

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
        eprintln!("The refactoring broke the code");
        return Err(RefactorStatusCodes::RefactoringProcucedBrokenCode as i32);
    }
    return Ok(());
    // TODO: output message / status that the code was broken after refactoring
}