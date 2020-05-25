use rustc_hir::HirId;
use rustc_span::Span;

use crate::refactoring_invocation::{AstDiff, QueryResult, RefactoringErrorInternal, TyContext};
use crate::refactorings::visitors::{collect_local_variable_use, collect_struct_field_access_expressions};
use struct_constructor_call_collector::collect_struct_constructor_calls;
use struct_tuple_pattern_collector::collect_struct_tuple_patterns;

mod struct_constructor_call_collector;
mod struct_tuple_pattern_collector;

pub fn do_refactoring(tcx: &TyContext, struct_hir_id: HirId, field_index: usize, field_ty_span: Span) -> QueryResult<AstDiff> {

    let struct_patterns = collect_struct_tuple_patterns(tcx, struct_hir_id, field_index); 

    if !struct_patterns.other.is_empty() {
        return Err(RefactoringErrorInternal::used_in_pattern(&field_index.to_string()));
    }

    let mut changes = vec![tcx.map_change(
        field_ty_span,
        format!("Box<{}>", tcx.get_source(field_ty_span))
    )?];

    for struct_expression in collect_struct_constructor_calls(tcx, struct_hir_id, field_index) {
        let replacement = format!("Box::new({})", tcx.get_source(struct_expression));
        changes.push(tcx.map_change(struct_expression, replacement)?);
    }

    for field_access_expression in collect_struct_field_access_expressions(tcx, struct_hir_id, &field_index.to_string()) {
        let replacement = format!("(*{})", tcx.get_source(field_access_expression));
        changes.push(tcx.map_change(field_access_expression, replacement)?);
    }

    for new_binding in struct_patterns.new_bindings {
        for local_use in collect_local_variable_use(tcx.0, new_binding) {
            let replacement = format!("(*{})", tcx.get_source(local_use));
            changes.push(tcx.map_change(local_use, replacement)?);
        }
    }

    Ok(AstDiff(changes))
}