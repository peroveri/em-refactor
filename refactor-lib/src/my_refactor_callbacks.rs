use crate::change::{Change, FileReplaceContent};
use crate::refactor_definition::{InternalErrorCodes, RefactorDefinition, RefactoringError};
use crate::refactorings::{do_after_expansion_refactoring, do_ty_refactoring, is_after_expansion_refactoring};
use rustc::ty;
use rustc_driver;
use rustc_interface::interface;
use rustc_span::FileName;
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
    pub content: Option<String>, // TODO: remove content and multiple_files fields
    pub multiple_files: bool,
    pub ignore_missing_file: bool,
    pub file_replace_content: Vec<FileReplaceContent>
}

impl MyRefactorCallbacks {
    pub fn from_arg(arg: RefactorDefinition) -> MyRefactorCallbacks {
        MyRefactorCallbacks {
            args: arg,
            result: Err(RefactoringError::new(InternalErrorCodes::Error, "".to_owned())), // shouldnt be Err by default, but something like None
            content: None,
            multiple_files: false,
            ignore_missing_file: false,
            file_replace_content: vec![]
        }
    }

    fn output_changes(&mut self, tcx: ty::TyCtxt, changes: &[Change]) {
        if changes.is_empty() {
            return;
        }
        self.map_file_replacements2(tcx, changes);
        self.multiple_files = contains_multiple_files(changes);
        if self.multiple_files {
            return;
        }
        
        let mut changes = changes.to_owned();
        changes.sort_by_key(|c| c.start);
        changes.reverse();
        let source_map = tcx.sess.source_map();
        let file_name = FileName::Real(PathBuf::from(changes[0].file_name.to_string()));
        let source_file = source_map.get_source_file(&file_name).unwrap();
        let mut content = if let Some(s) = &source_file.src {
            s.to_string()
        } else {
            panic!("")
        };

        for change in &changes {
            let s1 = &content[..(change.start) as usize];
            let s2 = &content[(change.end) as usize..];
            content = format!("{}{}{}", s1, change.replacement, s2);

        }

        self.content = Some(content);
    }


    fn map_file_replacements2(&mut self, tcx: ty::TyCtxt, changes: &[Change]) {
        for change in changes {
            let replacement = self.map_file_replacements(tcx, &change);
            self.file_replace_content.push(replacement);
        }
    }

    fn map_file_replacements(&mut self, tcx: ty::TyCtxt, change: &Change) -> FileReplaceContent {
        let range = crate::refactor_definition::SourceCodeRange {
            file_name: change.file_name.to_string(),
            from: change.start,
            to: change.end
        };

        let span = crate::refactorings::utils::map_range_to_span(tcx, &range).unwrap();

        let lines = tcx.sess.source_map().span_to_lines(span).unwrap().lines;
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

pub fn serialize_file_replacements(replacements: &Vec<FileReplaceContent>) ->  Result<String, i32> {
    if let Ok(serialized) = serde_json::to_string(replacements) {
        Ok(serialized)
    } else {
        Err(4)
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
            // rustc_driver::Compilation::Stop
        } else {
        }
        rustc_driver::Compilation::Continue
    }
    fn after_analysis<'tcx>(
        &mut self, 
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {

            if let Ok(changes) = self.result.clone() {
                self.output_changes(tcx, &changes);
                return;
            }

            self.result = do_ty_refactoring(tcx, &self.args);
            if let Ok(changes) = self.result.clone() {
                self.output_changes(tcx, &changes);
            }
        });
        rustc_driver::Compilation::Stop
    }
}

fn contains_multiple_files(changes: &[Change]) -> bool {
    use std::collections::HashSet;
    use std::iter::FromIterator;
    let set: HashSet<String> = HashSet::from_iter(
        changes
            .iter()
            .map(|c| c.file_name.to_string())
            .collect::<Vec<_>>(),
    );
    set.len() > 1
}
