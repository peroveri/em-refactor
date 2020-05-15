use rustc_interface::{interface, Queries, interface::Compiler};
use rustc_middle::ty::TyCtxt;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use quote::__private::TokenStream;
use rustc_span::{BytePos, Span};
use tempfile::TempDir;
use crate::refactoring_invocation::{argument_list_to_refactor_def, AstContext, get_sys_root, MyRefactorCallbacks, Query, QueryResult, TyContext};
use refactor_lib_types::{FileStringReplacement, RefactorArgs, SelectionType};

/**
 * Function that can be used to run unit tests.
 * Accepts a TokenStream (from quote) and a function with a single parameter TyCtxt.
 */
#[allow(unused)]
pub fn run_after_analysis<F>(program: TokenStream, func: F)
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
pub fn run_after_expansion<F>(program: TokenStream, func: F)
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
pub fn run_after_parsing<F>(program: TokenStream, func: F)
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
#[derive(Clone)]
pub struct TestInit {
    add_comment: bool,
    program: String,
    refactoring: String,
    selection_type: SelectionType,
    with_changes: Vec<Vec<FileStringReplacement>>
}
impl TestInit {
    pub fn from_refactoring(program: &str, refactoring: &str) -> Self {
        Self {
            add_comment: false,
            program: program.to_string(),
            refactoring: refactoring.to_string(),
            selection_type: SelectionType::Comment("test-id".to_string()),
            with_changes: vec![]
        }
    }
    pub fn with_add_comment(&self) -> Self {
        let mut ret = self.clone();
        ret.add_comment = true;
        ret
    }
    pub fn with_changes(&self, changes: Vec<Vec<FileStringReplacement>>) -> Self {
        let mut ret = self.clone();
        ret.with_changes = changes;
        ret
    }
}
pub fn run_refactoring(init: TestInit) -> QueryResult<String>  {
    let (rustc_args, d) = init_main_rs_and_get_args(&init.program);
    let q = argument_list_to_refactor_def(
        RefactorArgs {
            file: format!("{}", d.path().join("./main.rs").to_str().unwrap().to_owned()),
            refactoring: init.refactoring.to_string(),
            selection: init.selection_type,
            unsafe_: false,
            deps: vec![],
            add_comment: init.add_comment,
            with_changes: init.with_changes
        }
    )?;

    let mut c = MyRefactorCallbacks::from_arg(q, false);
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();
    Ok(crate::refactoring_invocation::get_file_content(&c.result?.0).unwrap())
}
pub struct TestContext {
    pub main_path: String,
    pub selection: Option<(u32, u32)>
}
fn find_selection(program: &str) -> Option<(u32, u32)> {
    const S0_STR: &str = "/*START*/";
    const S1_STR: &str = "/*END*/";

    if let Some(x0) = program.find(S0_STR) {
        Some((
            (x0 + S0_STR.len()) as u32, 
            program.find(S1_STR).unwrap() as u32))
    } else {
        None
    }
}
pub fn assert_success5<T>(prog: &str, init: Box<dyn Fn(&TestContext) -> Box<dyn Fn(&AstContext) -> QueryResult<T> + Send>>, expected: QueryResult<T>) 
    where T: std::fmt::Debug + PartialEq + Send {

    let (rustc_args, d) = init_main_rs_and_get_args(&format!("{}", prog));
    let test_context = TestContext {
        main_path: d.path().join("./main.rs").to_str().unwrap().to_owned(),
        selection: None
    };
    let q = init(&test_context);

    let mut c = MyRefactorCallbacks::from_arg(Query::AfterExpansion(q), false);
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    d.close().unwrap();
    err.unwrap();

    assert_eq!(c.result, expected);
}
pub fn run_ast_query<T, F>(program: &str, init: F) -> QueryResult<T>
    where
        F: Fn(TestContext) -> Box<dyn Fn(&AstContext) -> QueryResult<T> + Send>,
        T: std::fmt::Debug + PartialEq + Send {
    
    let (rustc_args, d) = init_main_rs_and_get_args(program);
    let ctx = TestContext {
        main_path: d.path().join("./main.rs").to_str().unwrap().to_owned(),
        selection: find_selection(program)
    };
    let q = init(ctx);

    let mut c = MyRefactorCallbacks::from_arg(Query::AfterExpansion(q), false);
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();

    c.result
}
pub fn run_ty_query<T, F>(program: &str, init: F) -> QueryResult<T>
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

    let mut c = MyRefactorCallbacks::from_arg(Query::AfterParsing(q), false);
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();

    c.result
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

    let mut c = MyRefactorCallbacks::from_arg(Query::AfterParsing(q), false);
    let err = rustc_driver::run_compiler(&rustc_args, &mut c, None, None);
    err.unwrap();

    assert_eq!(c.result.unwrap(), expected);
}