use crate::change::Change;
use crate::refactor_args::RefactorArgs;
use crate::refactorings::expr_use_visit::CollectVarsArgs;
use crate::refactorings::stmts_visitor::visit_stmts;
use rustc::hir;
use rustc::ty;
use syntax::source_map::{BytePos, Span};

fn get_selection(s: &str) -> (u32, u32) {
    let vs = s.split(':').collect::<Vec<_>>();
    (vs[0].parse().unwrap(), vs[1].parse().unwrap())
}

fn get_stmts_source(source_map: &syntax::source_map::SourceMap, stmts: &[&hir::Stmt]) -> String {
    stmts
        .iter()
        .map(|stmt| source_map.span_to_snippet(stmt.span).unwrap().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn map_to_span(
    source_map: &syntax::source_map::SourceMap,
    selection: (u32, u32),
    file: &str,
) -> Span {
    let filename = syntax::source_map::FileName::Real(std::path::PathBuf::from(file));
    let source_file = source_map.get_source_file(&filename).unwrap();
    Span::with_root_ctxt(
        BytePos(selection.0 + source_file.start_pos.0),
        BytePos(selection.1 + source_file.start_pos.0),
    )
}

/**
 * WIP
 *
 * Extract method (from statements)
 *
 * Input:
 * - F - A function
 * - M - The module containing F
 * - S - A selection in F of consecutive statements
 *
 * Assumptions:
 * - F is not a method
 *
 * Steps:
 * G <- new function with fresh name
 * add G to M
 * Vb <- variables in S declared before S
 * Va <- variables in S declared in S and used after S
 * for each V in Vb
 *   add V as parameter of G
 * for each V in Va
 *   add V as return type of G
 * move S to G
 * replace S in F with call to G
 * if |Va| > 0
 *   add return statement at the end of M with Va
 *   add declaration for all Va's before call to G and assign
 * 
 */

pub fn do_refactoring(ty: ty::TyCtxt, args: &RefactorArgs) -> Vec<Change> {
    let spi = map_to_span(
        ty.sess.source_map(),
        get_selection(&args.selection),
        &args.file,
    );

    let stmts_visit_res = visit_stmts(ty, spi);

    if let Some(stmts) = stmts_visit_res {
        let collect_args = CollectVarsArgs {
            body_id: stmts.fn_body_id,
            spi,
        };
        let vars_used = crate::refactorings::expr_use_visit::collect_vars(ty, collect_args);

        if vars_used.get_return_values().len() > 1 {
            return vec![]; // TODO: should be error
        }

        let params = vars_used
            .get_arguments()
            .iter()
            .map(|arg| arg.as_param())
            .collect::<Vec<_>>()
            .join(", ");

        let new_fn = format!(
            "fn {}({}) {{\n{}\n}}\n",
            args.new_function,
            params,
            get_stmts_source(ty.sess.source_map(), &stmts.S)
        );

        let arguments = vars_used.get_arguments().iter().map(|arg| arg.as_arg()).collect::<Vec<_>>().join(", ");

        let fn_call = format!("{}({});", args.new_function, arguments);
        let si_start = stmts.S.first().unwrap().span.lo().0;
        let si_end = stmts.S.last().unwrap().span.hi().0;

        vec![Change {
            file_name: args.file.to_string(),
            start: stmts.fn_decl_pos,
            end: stmts.fn_decl_pos,
            replacement: new_fn,
        },
        Change {
            file_name: args.file.to_string(),
            start: si_start,
            end: si_end,
            replacement: fn_call
        }]
    } else {
        println!("no statements");
        vec![]
    }
}


/*
 * For each variable used in S and declared before
 *   if consumed, it must be moved into the function
 *   if mutated, it must be passed as mutable
 *   if used later, it must be borrowed
 * 
 * For each variable declared in S and used later
 *   if it is a borrow, fail?
 *   must be returned
 */
fn create_argument_list() {

}