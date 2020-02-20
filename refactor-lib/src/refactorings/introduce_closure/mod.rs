use super::utils::{map_change_from_span, get_source};
use crate::refactoring_invocation::{FileStringReplacement, RefactoringErrorInternal};
use block_collector::collect_block;
use cf_expr_collector::{collect_cfs};
use rustc::ty::TyCtxt;
use rustc_span::Span;
use cf_collection::*;

mod block_collector;
mod cf_collection;
mod cf_expr_collector;

fn get_call(tcx: TyCtxt, span: Span) -> FileStringReplacement {
    map_change_from_span(tcx.sess.source_map(), span, format!("(|| {})()", get_source(tcx, span)))
}

/// 
/// ## Algorithm
/// 
/// Input
/// - Block
/// 
/// Output
/// - A new expression containing the block as an anonymous closure
/// 
/// Preconditions
/// - Break, continue, return, `?` are not currently handled, so they must be preventet
/// 
pub fn do_refactoring(tcx: TyCtxt, span: Span) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {
    if let Some(result) = collect_block(tcx, span) {
        // option 1: the selection is just a block
        // option 2: the selection is an assignment where the rhs is a block

        // point of closure decl: immediately before this statement

        let cf_expr = collect_cfs(tcx, result.selected_block.hir_id);

        let replacements = 
        if cf_expr.has_cfs() {
            let block_src = get_source(tcx, span);
            let repl = cf_expr.replace_cfs(tcx, block_src, span.lo().0);
            let anon_inv = format!("match (|| {})() {{{}}}", repl, cf_expr.get_cf_arms());

            map_change_from_span(tcx.sess.source_map(), span, anon_inv)
        } else {
            get_call(tcx, result.selected_block.span)
        };



        Ok(vec![
            replacements,
        ])
        
    } else {
        Err(RefactoringErrorInternal::invalid_selection(span.lo().0, span.hi().0))
    }
}
