use rustc::ty::TyCtxt;
use rustc_interface::{interface, Queries, interface::Compiler};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use quote::__rt::TokenStream;
use rustc_span::{BytePos, Span};
use tempfile::TempDir;
use crate::refactoring_invocation::{argument_list_to_refactor_def, AstContext, get_sys_root, MyRefactorCallbacks, Query, QueryResult, RefactoringErrorInternal, TyContext};

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
    F: Fn(&Queries<'_>, &Compiler) -> (),
    F: Send,
{
    let (rustc_args, d) = init_main_rs_and_get_args(&format!("{}", program));
    let mut c = RustcAfterExpansionCallbacks(func);
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
    F: Fn(&Queries<'_>, &Compiler) -> (),
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
    F: Fn(&Queries<'_>, &Compiler) -> ()
{
    fn after_parsing<'tcx>(
        &mut self, 
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        self.0(queries, compiler);
        rustc_driver::Compilation::Stop
    }
}

struct RustcAfterExpansionCallbacks<F>(F);
impl<F> rustc_driver::Callbacks for RustcAfterExpansionCallbacks<F>
where
    F: Fn(&Queries<'_>, &Compiler) -> ()
{
    fn after_expansion<'tcx>(
        &mut self, 
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        self.0(queries, compiler);
        rustc_driver::Compilation::Stop
    }
}

pub fn init_main_rs_and_get_args(program: &str) -> (Vec<String>, TempDir)
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
pub fn assert_success(prog: TokenStream, refactoring: &str, span: (u32, u32), expected: &str)  {

    let program = &format!("{}", prog);
    
    let (rustc_args, d) = init_main_rs_and_get_args(&format!("{}", program));

    let q = argument_list_to_refactor_def(&vec![
        format!("--file={}", d.path().join("./main.rs").to_str().unwrap().to_owned()),
        format!("--selection={}:{}", span.0, span.1),
        format!("--refactoring={}", refactoring),
    ]).unwrap();

    let mut c = MyRefactorCallbacks::from_arg(q);
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();

    assert_eq!(crate::refactoring_invocation::get_file_content(&c.result.unwrap().0).unwrap(), expected);
}
pub fn assert_err(prog: quote::__rt::TokenStream, refactoring: &str, span: (u32, u32), expected: RefactoringErrorInternal)  {
    let program = &format!("{}", prog);
    
    let (rustc_args, d) = init_main_rs_and_get_args(&format!("{}", program));
    let q = argument_list_to_refactor_def(&vec![
        format!("--file={}", d.path().join("./main.rs").to_str().unwrap().to_owned()),
        format!("--selection={}:{}", span.0, span.1),
        format!("--refactoring={}", refactoring),
    ]).unwrap();

    let mut c = MyRefactorCallbacks::from_arg(q);
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();
    assert_eq!(c.result.unwrap_err(), expected);
}
pub fn assert_success2(prog: TokenStream, init: Box<dyn Fn(String) -> Box<dyn Fn(&AstContext) -> QueryResult<String> + Send>>, expected: &str)  {

    let program = &format!("{}", prog);
    
    let (rustc_args, d) = init_main_rs_and_get_args(&format!("{}", program));
    let q = init(d.path().join("./main.rs").to_str().unwrap().to_owned());

    let mut c = MyRefactorCallbacks::from_arg(Query::AfterExpansion(q));
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();

    assert_eq!(c.result.unwrap(), expected);
}
pub fn assert_err2(prog: TokenStream, init: Box<dyn Fn(String) -> Box<dyn Fn(&AstContext) -> QueryResult<String> + Send>>, expected: RefactoringErrorInternal)  {

    let program = &format!("{}", prog);
    
    let (rustc_args, d) = init_main_rs_and_get_args(&format!("{}", program));
    let q = init(d.path().join("./main.rs").to_str().unwrap().to_owned());

    let mut c = MyRefactorCallbacks::from_arg(Query::AfterExpansion(q));
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();

    assert_eq!(c.result.unwrap_err(), expected);
}
pub fn assert_success3<T, F>(program: &str, init: F, expected: T) 
    where
        F: Fn(String, u32, u32) -> Box<dyn Fn(&TyContext) -> QueryResult<T> + Send>,
        T: std::fmt::Debug + PartialEq + Send {
    const S0_STR: &str = "/*START*/";
    const S1_STR: &str = "/*END*/";
    let s0 = (program.find(S0_STR).unwrap() + S0_STR.len()) as u32;
    let s1 = program.find(S1_STR).unwrap() as u32;
    
    let (rustc_args, d) = init_main_rs_and_get_args(program);
    let main_path = d.path().join("./main.rs").to_str().unwrap().to_owned();
    let q = init(main_path, s0, s1);

    let mut c = MyRefactorCallbacks::from_arg(Query::AfterParsing(q));
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();

    assert_eq!(c.result.unwrap(), expected);
}