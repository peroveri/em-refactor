use crate::refactoring_invocation::{AstContext, InternalErrorCodes, Query, QueryResult, RefactoringErrorInternal, TyContext};
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::Queries;
use rustc_interface::interface::Compiler;

///
/// Handles callbacks from the compiler
/// after_parsing: AST
/// after_expansion: AST but macros have been expanded
/// after_analysis: HIR (desugared AST) after typechecking
///
pub struct MyRefactorCallbacks<T> {
    pub query: Query<T>,
    pub result: QueryResult<T>,
}

impl<T> MyRefactorCallbacks<T> {
    pub fn from_arg(q: Query<T>) -> Self {
        Self {
            query: q,
            result: Err(RefactoringErrorInternal::new(InternalErrorCodes::Error, "".to_owned())), // shouldnt be Err by default, but something like None
        }
    }
}

pub fn serialize<T>(t: &T) ->  Result<String, RefactoringErrorInternal>
    where T: serde::Serialize {
    if let Ok(serialized) = serde_json::to_string(t) {
        Ok(serialized)
    } else {
        Err(RefactoringErrorInternal::int("serialization failed"))
    }
}

impl<T> Callbacks for MyRefactorCallbacks<T> {
    fn after_expansion<'tcx>(
        &mut self, 
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>
    ) -> Compilation {

        if let Query::AfterExpansion(f) = &self.query {
            let mut ctx = AstContext::new(compiler, queries);
            ctx.load_crate();
            self.result = f(&ctx);
            Compilation::Stop
        } else {
            Compilation::Continue
        }
    }
    fn after_analysis<'tcx>(
        &mut self, 
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>
    ) -> Compilation {
        compiler.session().abort_if_errors();
        queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {

            if let Query::AfterParsing(f) = &self.query {
                let ctx = TyContext::new(tcx);
                self.result = f(&ctx);
            }
        });
        Compilation::Stop // TODO: if this is a local dependency, we should continue
    }
}
