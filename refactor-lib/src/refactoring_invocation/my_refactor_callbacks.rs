use crate::refactoring_invocation::{AstContext, InternalErrorCodes, FileStringReplacement,  RefactorDefinition, RefactoringErrorInternal, RefactorFail};
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
    pub args: RefactorDefinition,
    pub result: Result<Vec<FileStringReplacement>, RefactoringErrorInternal>,
    pub content: Option<String>, // TODO: remove content
}

impl MyRefactorCallbacks {
    pub fn from_arg(arg: RefactorDefinition) -> MyRefactorCallbacks {
        MyRefactorCallbacks {
            args: arg,
            result: Err(RefactoringErrorInternal::new(InternalErrorCodes::Error, "".to_owned())), // shouldnt be Err by default, but something like None
            content: None,
        }
    }

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
}

pub fn serialize<T>(t: &T) ->  Result<String, RefactorFail>
    where T: serde::Serialize {
    if let Ok(serialized) = serde_json::to_string(t) {
        Ok(serialized)
    } else {
        Err(RefactorFail::int("serialization failed"))
    }
}

fn do_after_exp(context: AstContext, args: &RefactorDefinition) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {
    match args {
        RefactorDefinition::PullUpItemDeclaration(range) => crate::refactorings::pull_up_item_declaration::do_refactoring(&context, context.map_range_to_span(range)?),
        _ => panic!()
    }
    
}

impl Callbacks for MyRefactorCallbacks {
    fn after_expansion<'tcx>(
        &mut self, 
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>
    ) -> Compilation {

        if let RefactorDefinition::PullUpItemDeclaration(..) = self.args {
            self.result = do_after_exp(AstContext::new(compiler, queries), &self.args);

            if let Ok(changes) = &self.result {
                self.content = Self::get_file_content(changes, compiler.session().source_map());
            }
            Compilation::Stop
        } else if is_after_expansion_refactoring(&self.args) {
            self.result = do_after_expansion_refactoring(&queries, compiler, &self.args);
            if let Ok(changes) = self.result.clone() {
                self.content = Self::get_file_content(&changes, compiler.session().source_map());
            }
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
            self.result = do_ty_refactoring(tcx, &self.args);
            if let Ok(changes) = self.result.clone() {
                self.content = MyRefactorCallbacks::get_file_content(&changes, tcx.sess.source_map());
            }
        });
        Compilation::Stop
    }
}
