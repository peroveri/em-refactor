use crate::refactoring_invocation::{AstContext, InternalErrorCodes, FileStringReplacement, Query, QueryResult, RefactorDefinition, RefactoringErrorInternal, RefactorFail};
use crate::refactorings::{do_after_expansion_refactoring, do_ty_refactoring, is_after_expansion_refactoring};
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::Queries;
use rustc_interface::interface::Compiler;
use rustc_span::FileName;
use rustc_span::source_map::SourceMap;
use std::path::PathBuf;

///
/// Handles callbacks from the compiler
/// after_parsing: AST
/// after_expansion: AST but macros have been expanded
/// after_analysis: HIR (desugared AST) after typechecking
///
pub struct MyRefactorCallbacks {
    pub args: Option<RefactorDefinition>,
    pub scr: Option<Query>,
    pub result: QueryResult<String>,
    pub old_result: Result<Vec<FileStringReplacement>, RefactoringErrorInternal>,
    pub content: Option<String>, // TODO: remove content
}

impl MyRefactorCallbacks {
    pub fn from_def(arg: RefactorDefinition) -> Self {
        Self {
            args: Some(arg),
            scr: None,
            old_result: Err(RefactoringErrorInternal::new(InternalErrorCodes::Error, "".to_owned())),
            result: Err(RefactoringErrorInternal::new(InternalErrorCodes::Error, "".to_owned())), // shouldnt be Err by default, but something like None
            content: None,
        }
    }
    
    pub fn from_arg(q: Query) -> Self {
        Self {
            args: None,
            scr: Some(q),
            old_result: Err(RefactoringErrorInternal::new(InternalErrorCodes::Error, "".to_owned())),
            result: Err(RefactoringErrorInternal::new(InternalErrorCodes::Error, "".to_owned())), // shouldnt be Err by default, but something like None
            content: None,
        }
    }

}

/// TODO: Return result
pub fn get_file_content(changes: &[FileStringReplacement], source_map: &SourceMap) -> Option<String> {
    let mut changes = changes.to_vec();
    changes.sort_by_key(|c| c.byte_start);
    changes.reverse();

    let file_name = FileName::Real(PathBuf::from(changes[0].file_name.to_string()));
    let source_file = source_map.get_source_file(&file_name).unwrap();
    let mut content = if let Some(s) = &source_file.src {
        s.to_string()
    } else {
        return None;
    };

    for change in &changes {
        let s1 = &content[..(change.byte_start) as usize];
        let s2 = &content[(change.byte_end) as usize..];
        content = format!("{}{}{}", s1, change.replacement, s2);
    }

    return Some(content);
}

pub fn serialize<T>(t: &T) ->  Result<String, RefactorFail>
    where T: serde::Serialize {
    if let Ok(serialized) = serde_json::to_string(t) {
        Ok(serialized)
    } else {
        Err(RefactorFail::int("serialization failed"))
    }
}

impl Callbacks for MyRefactorCallbacks {
    fn after_expansion<'tcx>(
        &mut self, 
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>
    ) -> Compilation {

        if let Some(Query::AfterExpansion(f)) = &self.scr {
            let mut ctx = AstContext::new(compiler, queries);
            ctx.load_crate();
            self.result = f.after_expansion(&ctx);
            Compilation::Stop
        } else if let Some(arg) = &self.args {
            if is_after_expansion_refactoring(&arg) {
                self.old_result = do_after_expansion_refactoring(&queries, compiler, arg);
                if let Ok(changes) = self.old_result.clone() {
                    self.content = get_file_content(&changes, compiler.session().source_map());
                }
                Compilation::Stop
            } else {
                Compilation::Continue
            }
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
            if let Some(arg) = &self.args {

                self.old_result = do_ty_refactoring(tcx, arg);
                if let Ok(changes) = self.old_result.clone() {
                    self.content = get_file_content(&changes, tcx.sess.source_map());
                }
            }
        });
        Compilation::Stop
    }
}
