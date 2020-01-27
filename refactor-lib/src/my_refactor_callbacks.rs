use crate::change::{Change, FileReplaceContent};
use crate::refactor_definition::{InternalErrorCodes, RefactorDefinition, RefactoringError, RefactorFail};
use crate::refactorings::{do_after_expansion_refactoring, do_ty_refactoring, is_after_expansion_refactoring};
use rustc_driver;
use rustc_interface::interface;
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
    pub result: Result<Vec<Change>, RefactoringError>,
    pub content: Option<String>, // TODO: remove content
    pub file_replace_content: Vec<FileReplaceContent>
}

impl MyRefactorCallbacks {
    pub fn from_arg(arg: RefactorDefinition) -> MyRefactorCallbacks {
        MyRefactorCallbacks {
            args: arg,
            result: Err(RefactoringError::new(InternalErrorCodes::Error, "".to_owned())), // shouldnt be Err by default, but something like None
            content: None,
            file_replace_content: vec![]
        }
    }

    pub fn get_file_content(changes: &[Change], source_map: &SourceMap) -> Option<String> {
        let mut changes = changes.to_vec();
        changes.sort_by_key(|c| c.start);
        changes.reverse();

        let file_name = FileName::Real(PathBuf::from(changes[0].file_name.to_string()));
        let source_file = source_map.get_source_file(&file_name).unwrap();
        let mut content = if let Some(s) = &source_file.src {
            s.to_string()
        } else {
            return None;
        };

        for change in &changes {
            let s1 = &content[..(change.start) as usize];
            let s2 = &content[(change.end) as usize..];
            content = format!("{}{}{}", s1, change.replacement, s2);
        }

        return Some(content);
    }


    fn map_file_replacements2(source_map: &SourceMap, changes: &[Change]) -> Vec<FileReplaceContent> {
        let mut changes = changes.to_vec();
        changes.sort_by_key(|c| c.start);
        changes.reverse();
        let mut ret = vec![];
        for change in changes.iter() {
            let replacement = Self::map_file_replacements(source_map, &change);
            ret.push(replacement);
        }
        return ret;
    }

    fn map_file_replacements(source_map: &SourceMap, change: &Change) -> FileReplaceContent {
        let range = crate::refactor_definition::SourceCodeRange {
            file_name: change.file_name.to_string(),
            from: change.start,
            to: change.end
        };
        let span = crate::refactorings::utils::map_range_to_span(source_map, &range).unwrap();

        let lines = source_map.span_to_lines(span).unwrap().lines;
        let line_start = lines.first().unwrap();
        let line_end = lines.last().unwrap();
        FileReplaceContent {
            byte_end: change.end,
            byte_start: change.start,
            char_end: line_end.end_col.0,
            char_start: line_start.start_col.0,
            file_name: change.file_name.to_string(),
            line_end: line_end.line_index,
            line_start: line_start.line_index,
            replacement: change.replacement.to_string()
        }
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

impl rustc_driver::Callbacks for MyRefactorCallbacks {
    fn after_expansion<'tcx>(
        &mut self, 
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {
        if is_after_expansion_refactoring(&self.args) {
            self.result = do_after_expansion_refactoring(&queries, compiler, &self.args);
            if let Ok(changes) = self.result.clone() {
                self.content = Self::get_file_content(&changes, compiler.session().source_map());
                self.file_replace_content = Self::map_file_replacements2(compiler.session().source_map(), &changes);
            }
            rustc_driver::Compilation::Stop
        } else {
            rustc_driver::Compilation::Continue
        }
    }
    fn after_analysis<'tcx>(
        &mut self, 
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            self.result = do_ty_refactoring(tcx, &self.args);
            if let Ok(changes) = self.result.clone() {
                self.content = MyRefactorCallbacks::get_file_content(&changes, tcx.sess.source_map());
                self.file_replace_content = Self::map_file_replacements2(tcx.sess.source_map(), &changes);
            }
        });
        rustc_driver::Compilation::Stop
    }
}
