use rustc_hir::HirId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::output_types::FileStringReplacement;
use crate::refactoring_invocation::RefactoringErrorInternal;
use crate::refactorings::utils::{get_source, map_change_from_span};
use crate::refactorings::visitors::{collect_local_variable_use, collect_struct_field_access_expressions};
use struct_constructor_call_collector::collect_struct_constructor_calls;
use struct_tuple_pattern_collector::collect_struct_tuple_patterns;

mod struct_constructor_call_collector;
mod struct_tuple_pattern_collector;

pub fn do_refactoring(tcx: TyCtxt, struct_hir_id: HirId, field_index: usize, field_ty_span: Span) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {

    let struct_patterns = collect_struct_tuple_patterns(tcx, struct_hir_id, field_index); 

    if !struct_patterns.other.is_empty() {
        return Err(RefactoringErrorInternal::used_in_pattern(&field_index.to_string()));
    }
    let source_map = tcx.sess.source_map();

    let mut changes = vec![map_change_from_span(
        source_map,
        field_ty_span,
        format!("Box<{}>", get_source(tcx, field_ty_span)),
    )];

    for struct_expression in collect_struct_constructor_calls(tcx, struct_hir_id, field_index) {
        let replacement = format!("Box::new({})", get_source(tcx, struct_expression));
        changes.push(map_change_from_span(source_map, struct_expression, replacement));
    }

    for field_access_expression in collect_struct_field_access_expressions(tcx, struct_hir_id, &field_index.to_string()) {
        let replacement = format!("(*{})", get_source(tcx, field_access_expression));
        changes.push(map_change_from_span(source_map, field_access_expression, replacement));
    }

    for new_binding in struct_patterns.new_bindings {
        for local_use in collect_local_variable_use(tcx, new_binding) {
            let replacement = format!("(*{})", get_source(tcx, local_use));
            changes.push(map_change_from_span(source_map, local_use, replacement));
        }
    }

    Ok(changes)

}