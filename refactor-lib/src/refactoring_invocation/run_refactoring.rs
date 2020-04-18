use refactor_lib_types::{FileStringReplacement};
use crate::refactoring_invocation::{argument_list_to_refactor_def, AstDiff, from_error, from_success, MyRefactorCallbacks, QueryResult, RefactoringErrorInternal, rustc_rerun, serialize, should_run_rustc_again};

pub fn run_refactoring_and_output_result(refactor_args: Vec<String>, rustc_args: Vec<String>) -> Result<(), i32> {
    
    match run_refactoring(&refactor_args, &rustc_args) {
        Err(err) => {
            if refactor_args.contains(&"--output-replacements-as-json".to_owned()) {
                println!("{}", serialize(&from_error(&rustc_args, err)).unwrap());
                Ok(())
            } else {
                eprintln!("{}", err.message);
                Err(-1)
            }
        },
        Ok(astdiff) => {
            if refactor_args.contains(&"--output-replacements-as-json".to_owned()) {
                print!("{}", serialize(&from_success(&rustc_args, astdiff.0)).unwrap());
            } else {
                print!("{}", get_file_content(&astdiff.0).unwrap());
            }
            Ok(())
        }
    }

}

fn run_refactoring(refactor_args: &Vec<String>, rustc_args: &Vec<String>) -> QueryResult<AstDiff> {


    // 1. Run refactoring callbacks
    let refactor_res = run_refactoring_internal(rustc_args, refactor_args)?;

    // 2. Rerun the compiler to check if any errors were introduced
    // Runs with default callbacks
    if should_run_rustc_again(refactor_args) && !refactor_res.0.is_empty() {
        rustc_rerun(&refactor_res.0, &rustc_args)?;
    }

    Ok(refactor_res)
}

fn run_refactoring_internal(rustc_args: &[String], refactor_args: &[String]) -> QueryResult<AstDiff> {
    
    let refactor_def = argument_list_to_refactor_def(refactor_args)?;

    let mut my_refactor = MyRefactorCallbacks::from_arg(refactor_def);

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
        return Err(RefactoringErrorInternal::compile_err());
    }
    my_refactor.result
}


pub fn get_file_content(changes: &[FileStringReplacement]) -> Option<String> {
    use std::fs::File;
    use std::io::prelude::*;
    let mut changes = changes.to_vec();
    changes.sort_by_key(|c| c.byte_start);
    changes.reverse();

    let mut file = File::open(&changes[0].file_name).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    
    for change in &changes {
        let s1 = &content[..(change.byte_start) as usize];
        let s2 = &content[(change.byte_end) as usize..];
        content = format!("{}{}{}", s1, change.replacement, s2);
    }

    return Some(content);
}