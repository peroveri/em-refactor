use rustc::ty::TyCtxt;
use rustc_interface::{interface, Queries};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use rustc_span::{BytePos, Span};
use tempfile::TempDir;
use crate::refactoring_invocation::get_sys_root;

/**
 * Function that can be used to run unit tests.
 * Accepts a TokenStream (from quote) and a function with a single parameter TyCtxt.
 */
#[allow(unused)]
pub fn run_after_analysis<F>(program: quote::__rt::TokenStream, func: F)
where
    F: Fn(TyCtxt<'_>) -> (),
    F: Send
{
    let (rustc_args, d) = init_main_rs_and_get_args(&format!("{}", program));
    let mut c = RustcAfterAnalysisCallbacks(func);
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();
}
/**
 * Function that can be used to run unit tests.
 * Accepts a TokenStream (from quote) and a function with a single parameter TyCtxt.
 */
#[allow(unused)]
pub fn run_after_expansion<F>(program: quote::__rt::TokenStream, func: F)
where
    F: Fn(&Queries<'_>) -> (),
    F: Send,
{
    let (rustc_args, d) = init_main_rs_and_get_args(&format!("{}", program));
    let mut c = RustcAfterParsingCallbacks(func);
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();
}

/**
 * Function that can be used to run unit tests.
 * Accepts a TokenStream (from quote) and a function with a single parameter TyCtxt.
 */
#[allow(unused)]
pub fn run_after_parsing<F>(program: quote::__rt::TokenStream, func: F)
where
    F: Fn(&Queries<'_>) -> (),
    F: Send,
{
    let (rustc_args, d) = init_main_rs_and_get_args(&format!("{}", program));
    let mut c = RustcAfterParsingCallbacks(func);
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();
}

pub fn create_test_span(lo: u32, hi: u32) -> rustc_span::Span {
    Span::with_root_ctxt(BytePos(lo), BytePos(hi))
}

struct RustcAfterAnalysisCallbacks<F>(F);

impl<F> rustc_driver::Callbacks for RustcAfterAnalysisCallbacks<F>
where
    F: Fn(TyCtxt<'_>) -> ()
{
    fn after_analysis<'tcx>(
        &mut self, 
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        queries
            .global_ctxt()
            .unwrap()
            .peek_mut()
            .enter(|tcx| self.0(tcx));
        rustc_driver::Compilation::Stop
    }
}

struct RustcAfterParsingCallbacks<F>(F);
impl<F> rustc_driver::Callbacks for RustcAfterParsingCallbacks<F>
where
    F: Fn(&Queries<'_>) -> ()
{
    fn after_parsing<'tcx>(
        &mut self, 
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        self.0(queries);
        rustc_driver::Compilation::Stop
    }
}

struct RustcAfterExpansionCallbacks<F>(F);
impl<F> rustc_driver::Callbacks for RustcAfterExpansionCallbacks<F>
where
    F: Fn(&Queries<'_>) -> ()
{
    fn after_expansion<'tcx>(
        &mut self, 
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        self.0(queries);
        rustc_driver::Compilation::Stop
    }
}

fn init_main_rs_and_get_args(program: &str) -> (Vec<String>, TempDir)
{
    let tmp_dir = TempDir::new().unwrap_or_else(|_| panic!("failed to create tmp dir"));
    let tmp_dir_path = tmp_dir.path();
    set_main_rs(tmp_dir_path, program).unwrap_or_else(|_| panic!("failed to set main rs"));

    (vec![
        ".".to_owned(),
        "--sysroot".to_owned(),
        get_sys_root(),
        tmp_dir_path.join("main.rs").to_str().unwrap().to_string(),
        "--allow".to_owned(),
        "dead_code".to_owned(),
        "--allow".to_owned(),
        "deprecated".to_owned(),
        "--allow".to_owned(),
        "unused".to_owned(),
        "--crate-type".to_owned(),
        "lib".to_owned(),
        format!("--out-dir={}", tmp_dir_path.to_str().unwrap()),
    ], tmp_dir)
}
fn set_main_rs(path: &Path, content: &str) -> std::io::Result<()> {
    let path = path.join("./main.rs");
    assert!(
        !path.exists(),
        "main.rs already existed: {}",
        path.to_str().unwrap()
    );
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
