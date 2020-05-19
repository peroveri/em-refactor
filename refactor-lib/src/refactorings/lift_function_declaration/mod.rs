use rustc_span::Span;
use crate::refactoring_invocation::{AstDiff, QueryResult, TyContext};
use crate::refactorings::visitors::hir::{collect_function_definition, FnDefinition};
use qpath_res_collector::collect_qpaths;

mod qpath_res_collector;

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

    match fn_def.impl_ {
        Some((_, false)) => move_to_impl(tcx, fn_def),
        _ => move_to_parent_mod(tcx, fn_def)
    }
}

fn move_to_parent_mod(tcx: &TyContext, fn_def: FnDefinition) -> QueryResult<AstDiff> {

    let changes = vec![
        tcx.map_change(fn_def.span, "".to_owned())?,
        tcx.map_change(fn_def.get_parent_mod_inner().shrink_to_hi(), format!("\n{}", tcx.get_source(fn_def.span)))?
    ];

    Ok(AstDiff(changes))
}
fn move_to_impl(tcx: &TyContext, fn_def: FnDefinition) -> QueryResult<AstDiff> {

    let mut changes = vec![
        tcx.map_change(fn_def.span, "".to_owned())?,
        tcx.map_change(fn_def.impl_.unwrap().0.shrink_to_hi(), format!("\n{}", tcx.get_source(fn_def.span)))?
    ];

    for span in collect_qpaths(tcx, fn_def.hir_id) {
        changes.push(
            tcx.map_change(span, format!("Self::{}", tcx.get_source(span)))?
        );
    }

    Ok(AstDiff(changes))
}
