#![feature(rustc_private)]

// Need to add compiler dependencies, as they are not listed in Cargo.toml
extern crate rustc;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_span;
extern crate rustc_typeck;
extern crate syntax;

use std::process::exit;
use refactoring_invocation::{pass_to_rustc, run_refactoring, rustc_rerun, should_pass_to_rustc, should_run_rustc_again};
use arg_mappings::{arg_value, get_compiler_args, get_refactor_args};
use change::map_success_to_output;

mod change;
mod extra;
mod my_refactor_callbacks;
mod refactor_definition;
mod refactor_definition_parser;
mod refactorings;
mod refactoring_invocation;
mod rustc_utils;
mod arg_mappings;

#[cfg(test)]
mod test_utils;
#[cfg(test)]
use test_utils::{create_test_span, run_after_analysis/*, run_after_expansion, run_after_parsing*/};

pub enum RefactorStatusCodes {
    Success = 0,
    InputDoesNotCompile = 1,
    RefactoringProcucedBrokenCode = 2,
    BadFormatOnInput = 3,
    // Serializing = 4,
    RustcPassFailed = 5,
    InternalRefactoringError = 6,
}

///
/// 1. Run rustc with refactoring callbacks
/// 2. Run rustc with no callbacks, but with changes applied by the refactorings
///
fn run_rustc() -> Result<(), i32> {
    // get compiler and refactoring args from input and environment
    let std_env_args = std::env::args().collect::<Vec<_>>();
    let rustc_args = get_compiler_args(&std_env_args);
    if should_pass_to_rustc(&rustc_args) {
        return pass_to_rustc(&rustc_args);
    }

    let refactor_args = get_refactor_args(&std_env_args);

    if extra::should_provide_type(&refactor_args) {
        return extra::provide_type(&refactor_args, &rustc_args);
    }

    let refactor_def = refactor_definition_parser::argument_list_to_refactor_def(&refactor_args);
    if let Err(err) = refactor_def {
        eprintln!("{}", err);
        return Err(RefactorStatusCodes::BadFormatOnInput as i32);
    }
    let refactor_def = refactor_def.unwrap();

    let refactor_res = run_refactoring(&rustc_args, refactor_def, &refactor_args);

    if let Err(err) = refactor_res {
        return Err(err);
    }
    let refactor_res = refactor_res.unwrap();
    if let None = refactor_res {
        return Ok(()); // special rule with --ignore-missing-file, but this should be removed
    }

    // 1. Run refactoring callbacks
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


pub fn main() {
    rustc_driver::init_rustc_env_logger();
    rustc_driver::install_ice_hook();
    exit(
        run_rustc()
            .err()
            .unwrap_or(RefactorStatusCodes::Success as i32),
    )
}
