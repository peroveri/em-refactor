use rustc::ty;
use rustc::hir;
use crate::refactorings::stmts_visitor::StmtsVisitor;
use crate::refactor_args::RefactorArgs;
use crate::change::Change;

/**
 * WIP
 * 
 * Extract method
 * 
 * Input:
 * - f - A function 
 * - m - The module containing f
 * - s - A selection in f (of consecutive statements?)
 * 
 * Assumptions:
 * - f is not a method
 * 
 * Steps:
 * g <- new function with fresh name
 * add g to m
 * vs <- all variables in s not declared in s
 * add vs as parameters of g
 * replace s with a call to g with arguments vs
 */

pub struct ExtractMethodRefactoring<'v> {
    pub tcx: ty::TyCtxt<'v>,
}
fn get_selection(s: &str) -> (u32, u32) {
    let vs = s.split(':').collect::<Vec<_>>();
    (vs[0].parse().unwrap(), vs[1].parse().unwrap())
}

fn get_selection_with_global_offset(source_map: &syntax::source_map::SourceMap, selection: (u32, u32), file: &str) -> (u32, u32) {
    let filename = syntax::source_map::FileName::Real(std::path::PathBuf::from(file));
    let source_file = source_map.get_source_file(&filename).unwrap();
    (selection.0 + source_file.start_pos.0, selection.1 + source_file.start_pos.0)
}

fn get_stmts_source(source_map: &syntax::source_map::SourceMap, stmts: &[&hir::Stmt]) -> String {
    stmts.iter().map(|stmt| source_map.span_to_snippet(stmt.span).unwrap().to_string()).collect::<Vec<_>>().join("\n")
}

impl<'v> ExtractMethodRefactoring<'v> {

    pub fn do_refactoring(ty: ty::TyCtxt, args: &RefactorArgs) -> Vec<Change> {
        let selection = get_selection_with_global_offset(ty.sess.source_map(), get_selection(&args.selection), &args.file);
        let stmts_visit = StmtsVisitor::visit(ty, &args.file, selection);

        if let Some(stmts) = stmts_visit.stmts {
            
            let new_fn = format!("fn {}() {{\n{}\n}}", args.new_function, get_stmts_source(ty.sess.source_map(), &stmts));

            vec![
                Change {
                    file_name: args.file.to_string(),
                    start: 0,
                    end: 0,
                    replacement: new_fn
                }
            ]
            // println!("{}", new_fn);
        } else {
            println!("no statements");
            vec![]
        }
    }
}
