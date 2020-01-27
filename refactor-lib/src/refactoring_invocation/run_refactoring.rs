use crate::change::{Change, map_success_to_output, FileReplaceContent};
use crate::my_refactor_callbacks;
use crate::refactor_definition::{RefactorDefinition, RefactoringError};
use crate::refactor_definition_parser::argument_list_to_refactor_def;
use crate::refactoring_invocation::{rustc_rerun, should_run_rustc_again};
use crate::RefactorStatusCodes;

pub fn run_refactoring(refactor_args: Vec<String>, rustc_args: Vec<String>) -> Result<(), i32> {

    let refactor_def =
    match argument_list_to_refactor_def(&refactor_args) {
        Err(err) =>  {
            eprintln!("{}", err);
            return Err(RefactorStatusCodes::BadFormatOnInput as i32);
        },
        Ok(v) => v
    };

    // 1. Run refactoring callbacks
    let refactor_res = run_refactoring_internal(&rustc_args, refactor_def, &refactor_args);

    if let Err(err) = refactor_res {
        // special rule with --ignore-missing-file, but this should be removed
        if err == RefactorStatusCodes::MissingFile as i32 {
            return Ok(());
        }
        return Err(err);
    }

    let (content, replacements, result) = refactor_res.unwrap();

    // 2. Rerun the compiler to check if any errors were introduced
    // Runs with default callbacks
    if should_run_rustc_again(&refactor_args) {
        let result = rustc_rerun(&result.unwrap(), &rustc_args);
        if result.is_err() {
            return result;
        }
    }

    if refactor_args.contains(&"--output-replacements-as-json".to_owned()) {
        print!("Crate:{}", my_refactor_callbacks::serialize(&map_success_to_output(&rustc_args, replacements))?);
    } else {
        print!("{}", content);
    }

    Ok(())
}

fn run_refactoring_internal(rustc_args: &[String], refactor_def: RefactorDefinition, refactor_args: &[String]) -> Result<(String, Vec<FileReplaceContent>, Result<Vec<Change>, RefactoringError>), i32> {

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
            return Err(RefactorStatusCodes::MissingFile as i32);
        }
        eprintln!("{}", err.message);
        return Err(RefactorStatusCodes::InternalRefactoringError as i32);
    }

    return Ok((content, replacements, my_refactor.result));
}
