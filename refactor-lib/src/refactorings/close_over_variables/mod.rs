use super::utils::{map_change_from_span, get_source};
use crate::refactoring_invocation::{AstDiff, QueryResult, TyContext};
use rustc_span::Span;
use expr_use_visit::collect_vars;
use crate::refactorings::visitors::hir::{collect_anonymous_closure, ExpressionUseKind};

mod expr_use_visit;
mod variable_use_collection;


/// Close Over Variables
/// 
/// ## Algorithm
/// 1. Convert selection (span) to method call expr M' with closure expr C' and argument list
/// 2. Collect variables Vs' used in C', declared outside
/// 3. For each V' in Vs'
///    a. Add V' as parameters of C'
///    b. Add V' as arguments of M'
///    c. If V' is a borrow, add deref to all occurences of V' in C'
pub fn do_refactoring(tcx: &TyContext, span: Span, _add_comment: bool) -> QueryResult<AstDiff> {
    let closure = collect_anonymous_closure(tcx, span)?;
    let vars = collect_vars(tcx.0, closure.body_id, tcx.get_body_span(closure.body_id))?;

    let mut changes = vec![];

    let params = vars.get_params_formatted();

    if params.len() > 0 {
        let params = if closure.has_params {
            format!(", {}", params)
        } else {
            params
        };
        changes.push(
            map_change_from_span(tcx.get_source_map(), closure.get_next_param_pos(), params)?);
    }

    let args = vars.get_args_formatted();
    if args.len() > 0 {
        let args = if closure.has_params {
            format!(", {}", args)
        } else {
            args
        };
        changes.push(map_change_from_span(tcx.get_source_map(), closure.get_next_arg_pos(), args.to_string())?);
    }

    for v in vars.get_borrows() {
        changes.push(map_change_from_span(tcx.get_source_map(), v, format!("(*{})", get_source(tcx.0, v)))?);
    }

    Ok(AstDiff(changes))
}
