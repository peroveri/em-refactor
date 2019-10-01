use rustc_interface::{interface};
use rustc_driver;
use rustc::ty;
use crate::refactorings::extract_method;
use crate::refactor_args::RefactorArgs;
use crate::change::Change;


pub struct MyRefactorCallbacks {
    pub args: RefactorArgs,
    pub changes: Vec<Change>
}

impl MyRefactorCallbacks {
    pub fn from_arg(arg: String) -> Result<MyRefactorCallbacks, String> {
        Ok(MyRefactorCallbacks {
            args: RefactorArgs::parse(arg)?,
            changes: vec![]
        })
    }
}

fn do_ty_refactoring(ty: ty::TyCtxt, args: &RefactorArgs) -> Vec<Change> {
    if args.refactoring == "extract-method" {
        extract_method::do_refactoring(ty, args)
    } else {
        vec![]
    }
}

impl rustc_driver::Callbacks for MyRefactorCallbacks {
    // fn after_expansion(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
    //     rustc_driver::Compilation::Continue
    // }
    fn after_analysis(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        compiler.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let changes = do_ty_refactoring(tcx, &self.args);
            
            output_changes(tcx, &changes);
        });
        rustc_driver::Compilation::Continue
    }
}

fn contains_multiple_files(changes: &[Change]) -> bool {
    use std::collections::HashSet;
    use std::iter::FromIterator;
    let set: HashSet<String> = HashSet::from_iter(changes.iter().map(|c| c.file_name.to_string()).collect::<Vec<_>>());
    set.len() > 1
}

fn output_changes(tcx: ty::TyCtxt, changes: &Vec<Change>) {
    if changes.is_empty() {
        return;
    }
    if contains_multiple_files(changes) {
        // TODO: figure out how the output should be
        panic!("changes in multiple files not currently supported");
    }
    let mut changes = changes.clone();
    changes.sort_by_key(|c| c.start);
    changes.reverse();
    
    let source_map = tcx.sess.source_map();
    let file_name = syntax::source_map::FileName::Real(std::path::PathBuf::from(changes[0].file_name.to_string()));
    let source_file = source_map.get_source_file(&file_name).unwrap();
    let mut content = if let Some(s) = &source_file.src { s.to_string()} else {panic!("")};

    let file_start_pos = source_file.start_pos.0 as u32;
    for change in &changes {
        let s1 = &content[..(change.start - file_start_pos) as usize];
        let s2 = &content[(change.end - file_start_pos) as usize..];
        content = format!("{}{}{}", s1, change.replacement, s2);
    }

    print!("{}", content);
}