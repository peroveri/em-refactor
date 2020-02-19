#![feature(rustc_private)]

// Need to add compiler dependencies, as they are not listed in Cargo.toml
extern crate rustc;
extern crate rustc_ast_pretty;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_span;
extern crate rustc_typeck;
extern crate syntax;

use std::process::exit;
use refactoring_invocation::{arg_value, get_compiler_args, get_refactor_args, pass_to_rustc, run_refactoring_and_output_result, should_pass_to_rustc};

mod extra;
mod refactorings;
mod refactoring_invocation;

#[cfg(test)]
mod test_utils;
#[cfg(test)]
use test_utils::{create_test_span, run_after_analysis/*, run_after_expansion*/, run_after_parsing};

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
    if extra::should_query_candidates(&refactor_args) {
        return extra::list_candidates(&refactor_args, &rustc_args);
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
