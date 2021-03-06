#![feature(rustc_private)]

// Need to add compiler dependencies, as they are not listed in Cargo.toml
extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_hir_pretty;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_session;
extern crate rustc_typeck;

pub mod candidates;
mod refactorings;
pub mod refactoring_invocation;

#[cfg(test)]
mod test_utils;
