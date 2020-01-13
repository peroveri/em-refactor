use super::utils::{get_file_offset, map_range_to_span};
use crate::change::Change;
use crate::refactor_definition::{RefactoringError, SourceCodeRange};
use expr_use_visit::{collect_vars, CollectVarsArgs};
use rustc::ty;
use stmts_visitor::visit_stmts;
use rustc_span::Span;
use rustc_span::source_map::SourceMap;

pub mod expr_use_visit;
mod stmts_visitor;

/**
 * rewrites: places in the source code where deref * needs to be added
 */
fn get_stmts_source(
    source_map: &SourceMap,
    span: Span,
    rewrites: &[u32],
) -> String {
    let mut source = source_map.span_to_snippet(span).unwrap();
    let mut rewrites = rewrites.to_owned();
    rewrites.sort();
    rewrites.reverse();

    for rewrite in rewrites {
        if span.lo().0 <= rewrite && rewrite <= span.hi().0 {
            let local_index = (rewrite - span.lo().0) as usize;
            source.insert(local_index, '*');
        }
    }

    source
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

pub fn do_refactoring(
    ty: ty::TyCtxt,
    range: &SourceCodeRange,
    new_function: &str,
) -> Result<Vec<Change>, RefactoringError> {
    let spi = map_range_to_span(ty, &range)?;
    let stmts_visit_res = visit_stmts(ty, spi);

    if let Some(stmts) = stmts_visit_res {
        let collect_args = CollectVarsArgs {
            body_id: stmts.fn_body_id,
            spi,
        };
        let vars_used = collect_vars(ty, collect_args);

        if vars_used.get_return_values().len() > 1 {
            return Err(RefactoringError::multiple_returnvalues());
        }

        let params = vars_used
            .get_arguments()
            .iter()
            .map(|arg| arg.as_param())
            .collect::<Vec<_>>()
            .join(", ");

        let new_fn = format!(
            "fn {}({}) {{\n{}\n}}\n",
            new_function,
            params,
            get_stmts_source(ty.sess.source_map(), spi, &vars_used.get_rewrites())
        );

        let arguments = vars_used
            .get_arguments()
            .iter()
            .map(|arg| arg.as_arg())
            .collect::<Vec<_>>()
            .join(", ");

        let fn_call = format!("{}({});", new_function, arguments);
        let si_start = stmts.stmts.first().unwrap().span.lo().0;
        let si_end = stmts.stmts.last().unwrap().span.hi().0;

        let file_offset = get_file_offset(ty, &range.file_name);

        Ok(vec![
            Change {
                file_name: range.file_name.to_string(),
                file_start_pos: file_offset,
                start: stmts.fn_decl_pos,
                end: stmts.fn_decl_pos,
                replacement: new_fn,
            },
            Change {
                file_name: range.file_name.to_string(),
                file_start_pos: file_offset,
                start: si_start,
                end: si_end,
                replacement: fn_call,
            },
        ])
    } else {
        Err(RefactoringError::invalid_selection(range.from, range.to))
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
