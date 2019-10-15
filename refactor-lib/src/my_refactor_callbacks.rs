use crate::change::Change;
use crate::refactor_definition::RefactorDefinition;
use crate::refactorings::do_ty_refactoring;
use rustc::ty;
use rustc_driver;
use rustc_interface::interface;
use std::path::PathBuf;
use syntax::source_map::FileName;

///
/// Handles callbacks from the compiler
/// after_parsing: AST
/// after_expansion: AST but macros have been expanded
/// after_analysis: HIR (desugared AST) after typechecking
///
pub struct MyRefactorCallbacks {
    pub args: RefactorDefinition,
    pub result: Result<Vec<Change>, String>,
    pub content: Option<String>,
}

impl MyRefactorCallbacks {
    pub fn from_arg(arg: RefactorDefinition) -> MyRefactorCallbacks {
        MyRefactorCallbacks {
            args: arg,
            result: Err("".to_owned()), // shouldnt be Err by default, but something like None
            content: None,
        }
    }

    fn output_changes(&mut self, tcx: ty::TyCtxt, changes: &[Change]) {
        if changes.is_empty() {
            return;
        }
        if contains_multiple_files(changes) {
            // TODO: figure out how the output should be
            panic!("changes in multiple files not currently supported");
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

        let file_start_pos = source_file.start_pos.0 as u32;
        for change in &changes {
            let s1 = &content[..(change.start - file_start_pos) as usize];
            let s2 = &content[(change.end - file_start_pos) as usize..];
            content = format!("{}{}{}", s1, change.replacement, s2);
        }

        self.content = Some(content);
    }
}

impl rustc_driver::Callbacks for MyRefactorCallbacks {
    // fn after_expansion(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
    //     rustc_driver::Compilation::Continue
    // }
    fn after_analysis(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        compiler.global_ctxt().unwrap().peek_mut().enter(|tcx| {
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
