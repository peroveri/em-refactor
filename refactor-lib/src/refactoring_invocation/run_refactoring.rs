use crate::output_types::{FileStringReplacement};
use crate::refactoring_invocation::{argument_list_to_refactor_def, from_error, from_success, map_args_to_query, MyRefactorCallbacks, RefactorFail, RefactoringErrorInternal, rustc_rerun, serialize, should_run_rustc_again};
use if_chain::if_chain;

pub fn run_refactoring_and_output_result(refactor_args: Vec<String>, rustc_args: Vec<String>) -> Result<(), i32> {
    
    match run_refactoring(&refactor_args, &rustc_args) {
        Err(e) => {
            eprintln!("{}", e.message);
            Err(-1)
        }, 
        Ok(RefactorResult::Success((content, replacements))) => {
            if refactor_args.contains(&"--output-replacements-as-json".to_owned()) {
                print!("Crate:{}", serialize(&from_success(&rustc_args, replacements)).unwrap());
            } else {
                print!("{}", content);
            }
            Ok(())
        }, 
        Ok(RefactorResult::Err(err)) => {
            if refactor_args.contains(&"--output-replacements-as-json".to_owned()) {
                println!("Crate:{}", serialize(&from_error(&rustc_args, err)).unwrap());
                Ok(())
            } else {
                eprintln!("{}", err.message);
                Err(-1)
            }
        }
    }

}

fn run_refactoring(refactor_args: &Vec<String>, rustc_args: &Vec<String>) -> Result<RefactorResult, RefactorFail> {


    // 1. Run refactoring callbacks
    let refactor_res = run_refactoring_internal(rustc_args, refactor_args)?;

    // 2. Rerun the compiler to check if any errors were introduced
    // Runs with default callbacks
    if_chain! {
        if let RefactorResult::Success((_, replacements)) = &refactor_res;
        if should_run_rustc_again(refactor_args) && !replacements.is_empty();
        then {
            rustc_rerun(&replacements, &rustc_args)?;
        }
    }

    Ok(refactor_res)
}

pub enum RefactorResult {
    Success((String, Vec<FileStringReplacement>)),
    Err(RefactoringErrorInternal)
}

fn run_refactoring_internal(rustc_args: &[String], refactor_args: &[String]) -> Result<RefactorResult, RefactorFail> {
    
    let mut my_refactor = if let Ok(r) = map_args_to_query(refactor_args) {
        MyRefactorCallbacks::from_arg(r)
    } else { // should be removed
        let refactor_def = argument_list_to_refactor_def(refactor_args)?;

        MyRefactorCallbacks::from_def(refactor_def)
    };

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

    if my_refactor.scr.is_some() {

        return match my_refactor.result {
            Err(err) => { Ok(RefactorResult::Err(err)) },
            Ok(astdiff) => {
                Ok(RefactorResult::Success((astdiff, vec![])))
            }
        };
    }

    match my_refactor.old_result {
        Err(err) => { Ok(RefactorResult::Err(err)) },
        Ok(replacements) => {
            Ok(RefactorResult::Success((my_refactor.content.unwrap_or("".to_owned()), replacements)))
        }
    }
}
