use crate::change::{map_success_to_output, FileReplaceContent};
use crate::my_refactor_callbacks;
use crate::refactor_definition::{RefactorDefinition, RefactorFail};
use crate::refactor_definition_parser::argument_list_to_refactor_def;
use crate::refactoring_invocation::{rustc_rerun, should_run_rustc_again};

pub fn run_refactoring(refactor_args: Vec<String>, rustc_args: Vec<String>) -> Result<(), RefactorFail> {

    let refactor_def = argument_list_to_refactor_def(&refactor_args)?;

    // 1. Run refactoring callbacks
    let refactor_res = run_refactoring_internal(&rustc_args, refactor_def, &refactor_args)?;

    let (content, replacements) = refactor_res;

    // 2. Rerun the compiler to check if any errors were introduced
    // Runs with default callbacks
    if should_run_rustc_again(&refactor_args) && !replacements.is_empty() {
        rustc_rerun(&replacements, &rustc_args)?;
    }

    if refactor_args.contains(&"--output-replacements-as-json".to_owned()) {
        print!("Crate:{}", my_refactor_callbacks::serialize(&map_success_to_output(&rustc_args, replacements))?);
    } else {
        print!("{}", content);
    }

    Ok(())
}

fn run_refactoring_internal(rustc_args: &[String], refactor_def: RefactorDefinition, refactor_args: &[String]) -> Result<(String, Vec<FileReplaceContent>), RefactorFail> {

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
        return Err(RefactorFail::compile_err());
    }
    let content = my_refactor.content.clone().unwrap_or_else(|| "".to_owned());
    let replacements = my_refactor.file_replace_content.clone();

    if let Err(err) = my_refactor.result {
        if err.code == crate::refactor_definition::InternalErrorCodes::FileNotFound &&
        refactor_args.contains(&"--ignore-missing-file".to_owned()) {
            return Ok((content, vec![]));
        }
        return Err(RefactorFail::int(&err.message));
    }

    return Ok((content, replacements));
}
