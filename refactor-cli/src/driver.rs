#![feature(rustc_private)]

// Need to add compiler dependencies, as they are not listed in Cargo.toml
extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_hir_pretty;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_typeck;

use std::process::exit;
use refactor_lib::candidates::query_candidates::list_candidates_and_print_result;
use refactor_lib::refactoring_invocation::{get_candidate_args, get_compiler_args, get_refactor_args, pass_to_rustc, run_refactoring_and_output_result, should_pass_to_rustc};

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

    if let Some(args) = get_candidate_args() {
        list_candidates_and_print_result(&args, &rustc_args);
        return Ok(());
    }
    run_refactoring_and_output_result(&get_refactor_args(), rustc_args)
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
