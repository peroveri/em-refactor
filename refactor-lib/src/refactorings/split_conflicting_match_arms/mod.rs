use super::utils::{get_source, get_struct_hir_id};
use super::visitors::collect_field;
use crate::output_types::FileStringReplacement;
use crate::refactoring_invocation::RefactoringErrorInternal;
use rustc::ty::TyCtxt;
use rustc_span::{Span};

mod conflicting_match_arm_collector;
mod expr_use_visit;
mod split_conflicting_match_arms_named;

/// Given a selection within a block, contiguous statements (0..n) and an expression (0|1)
/// It should pull up item declarations occuring at this block level
/// These item declarations can only be found in the selection of statements (if they are item decls.)
/// 
/// (Maybe?) Only if the item declarations are used outside the selection (before or after)
pub fn do_refactoring(tcx: TyCtxt, span: Span) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {
    if let Some((field, _index)) = collect_field(tcx, span) {
        let struct_hir_id = get_struct_hir_id(tcx, &field);
        if field.is_positional() {
            unimplemented!("split-conflicting-match-arms tuple");
        } else {
            split_conflicting_match_arms_named::do_refactoring(tcx, struct_hir_id, &field.ident.to_string(), field.ty.span)
        }
    } else {
        Err(RefactoringErrorInternal::invalid_selection_with_code(
            span.lo().0,
            span.hi().0,
            &get_source(tcx, span)
        ))
    }
}
