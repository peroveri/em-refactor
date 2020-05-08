use crate::refactoring_invocation::{AstDiff, QueryResult, TyContext};
use rustc_span::Span;
use crate::refactorings::visitors::hir::{collect_anonymous_closure};

mod expr_use_visit;
mod local_use_collector;
mod collect_new_closure;

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

    let new_closure = collect_new_closure::collect_vars3(tcx, closure.body_id)?;

    let mut changes = vec![];

    if new_closure.params.len() > 0 {
        let params = if closure.has_params {
            format!(", {}", new_closure.params)
        } else {
            new_closure.params.to_string()
        };
        changes.push(tcx.map_change(closure.get_next_param_pos(), params)?);
    }

    if new_closure.args.len() > 0 {
        let args = if closure.has_params {
            format!(", {}", new_closure.args)
        } else {
            new_closure.args.to_string()
        };
        changes.push(tcx.map_change(closure.get_next_arg_pos(), args)?);
    }
    
    for span in new_closure.uses {
        changes.push(tcx.map_change(span, format!("(*{})", tcx.get_source(span)))?);
    }
    for span in new_closure.selfs {
        changes.push(tcx.map_change(span, "self_".to_owned())?);
    }

    Ok(AstDiff(changes))
}
