use rustc::ty::TyCtxt;
use rustc_hir::HirId;
use rustc_span::Span;

use crate::change::Change;
use crate::refactor_definition::RefactoringError;
use crate::refactorings::utils::{get_source, map_change_from_span};
use super::local_variable_use_collector::collect_local_variable_use;
use super::struct_constructor_call_collector::collect_struct_constructor_calls;
use super::struct_field_access_expression_collector::collect_struct_field_access_expressions;
use super::struct_tuple_pattern_collector::collect_struct_tuple_patterns;


pub fn do_refactoring(tcx: TyCtxt, struct_hir_id: HirId, field_index: usize, field_ty_span: Span) -> Result<Vec<Change>, RefactoringError> {

    let struct_patterns = collect_struct_tuple_patterns(tcx, struct_hir_id, field_index); 

    if !struct_patterns.other.is_empty() {
        return Err(RefactoringError::used_in_pattern(&field_index.to_string()));
    }

    let constructor_calls = collect_struct_constructor_calls(tcx, struct_hir_id, field_index);
    
    let field_access_expressions = collect_struct_field_access_expressions(tcx, struct_hir_id, &field_index.to_string());
    let mut changes = vec![map_change_from_span(
        tcx,
        field_ty_span,
        format!("Box<{}>", get_source(tcx, field_ty_span)),
    )];

    for struct_expression in constructor_calls {
        let replacement = format!("Box::new({})", get_source(tcx, struct_expression));
        changes.push(map_change_from_span(tcx, struct_expression, replacement));
    }

    for field_access_expression in field_access_expressions {
        let replacement = format!("(*{})", get_source(tcx, field_access_expression));
        changes.push(map_change_from_span(tcx, field_access_expression, replacement));
    }

    for new_binding in struct_patterns.new_bindings {
        for local_use in collect_local_variable_use(tcx, new_binding) {
            let replacement = format!("(*{})", get_source(tcx, local_use));
            changes.push(map_change_from_span(tcx, local_use, replacement));
        }
    }

    Ok(changes)

}