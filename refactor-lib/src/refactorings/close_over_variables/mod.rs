use super::utils::{map_change_from_span, get_source};
use crate::output_types::FileStringReplacement;
use crate::refactoring_invocation::RefactoringErrorInternal;
use rustc::ty::TyCtxt;
use rustc_span::Span;
use anonymous_closure_collector::collect_anonymous_closure;
use expr_use_visit::collect_vars;

mod anonymous_closure_collector;
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
pub fn do_refactoring(tcx: TyCtxt, span: Span) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {
    if let Some(closure) = collect_anonymous_closure(tcx, span) {
        let source_map = tcx.sess.source_map();
        let vars = collect_vars(tcx, closure.body_id, closure.body_span);

        let mut changes = vec![];

        let params = vars.get_params().iter().map(|p| p.as_param()).collect::<Vec<_>>().join(", ");

        if params.len() > 0 {
            let params = if closure.has_params {
                format!(", {}", params)
            } else {
                params
            };
            changes.push(
                map_change_from_span(source_map, closure.params, params));
        }

        let args = vars.get_args().iter().map(|p| p.as_arg()).collect::<Vec<_>>().join(", ");
        if args.len() > 0 {
            let args = if closure.has_params {
                format!(", {}", args)
            } else {
                args
            };
            changes.push(map_change_from_span(source_map, closure.args, args.to_string()));
        }

        for v in vars.get_borrows() {
            changes.push(map_change_from_span(source_map, v, format!("(*{})", get_source(tcx, v))));
        }

        Ok(changes)
    } else {
        Err(RefactoringErrorInternal::invalid_selection_with_code(
            span.lo().0,
            span.hi().0,
            &get_source(tcx, span)
        ))
    }
}
