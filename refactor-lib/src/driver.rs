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
use refactoring_invocation::{pass_to_rustc, should_pass_to_rustc, run_refactoring_and_output_result};
use arg_mappings::{arg_value, get_compiler_args, get_refactor_args};

mod change;
mod extra;
mod my_refactor_callbacks;
mod refactor_definition;
mod refactor_definition_parser;
mod refactorings;
mod refactoring_invocation;
mod arg_mappings;

#[cfg(test)]
mod test_utils;
#[cfg(test)]
use test_utils::{create_test_span, run_after_analysis/*, run_after_expansion, run_after_parsing*/};

///
/// 1. Run rustc with refactoring callbacks
/// 2. Run rustc with no callbacks, but with changes applied by the refactorings
///
fn run_rustc() -> Result<(), i32> {
    // get compiler and refactoring args from input and environment
    let std_env_args = std::env::args().collect::<Vec<_>>();
    let rustc_args = get_compiler_args(&std_env_args);
    if should_pass_to_rustc(&rustc_args) {
        pass_to_rustc(&rustc_args);
        return Ok(());
    }

    let refactor_args = get_refactor_args(&std_env_args);

    if extra::should_provide_type(&refactor_args) {
        return extra::provide_type(&refactor_args, &rustc_args);
    }

    run_refactoring_and_output_result(refactor_args, rustc_args)
}

pub fn main() {
    rustc_driver::init_rustc_env_logger();
    rustc_driver::install_ice_hook();
    exit(
        run_rustc()
            .err()
            .unwrap_or(0),
    )
}
