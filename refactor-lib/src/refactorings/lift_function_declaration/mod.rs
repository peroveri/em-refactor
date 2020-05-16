use rustc_span::Span;
use crate::refactoring_invocation::{AstDiff, QueryResult, TyContext};
#[allow(unused)]
use call_expr_collector::collect_call_exprs;
use function_definition_collector::collect_function_definition;

mod call_expr_collector;
mod function_definition_collector;

/// Lift function declaration
pub fn do_refactoring(tcx: &TyContext, span: Span, _add_comment: bool) -> QueryResult<AstDiff> {

    // Find function declaration
    // If this should be a method
    // Find target (is it module or impl?)
    let fn_def = collect_function_definition(tcx, span)?;

    // if inside impl:
    //   move to impl, convert call_expr, maybe convert to method
    // else if inside impl_for:
    //   move to impl, convert call_expr, maybe create impl, maybe convert to method
    //   if create impl then we may have to add lifetime and gen args for impl, so skip this?
    // else:
    //   move to parent mod

    // also: for each call expr: replace ({path}) with path if it matches that

    let changes = vec![
        tcx.map_change(fn_def.span, "".to_owned())?,
        tcx.map_change(fn_def.get_parent_mod_inner().shrink_to_hi(), format!("\n{}", tcx.get_source(fn_def.span)))?
    ];

    // for call in collect_call_exprs(tcx, fn_def.hir_id) {

    // }

    Ok(AstDiff(changes))
}
