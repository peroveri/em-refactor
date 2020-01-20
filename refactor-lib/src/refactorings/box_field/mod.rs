use crate::change::Change;
use rustc::ty::TyCtxt;
use rustc_hir::{HirId, StructField};
use rustc_span::Span;

use crate::refactor_definition::RefactoringError;
use super::utils::get_source;
use struct_def_field_collector::collect_field;

mod box_named_field;
mod box_tuple_field;
mod local_variable_use_collector;
mod struct_constructor_call_collector;
mod struct_def_field_collector;
mod struct_expression_collector;
mod struct_field_access_expression_collector;
mod struct_named_pattern_collector;
mod struct_tuple_pattern_collector;

pub struct StructPatternCollection {
    pub new_bindings: Vec<HirId>,
    pub other: Vec<Span>
}

pub fn get_struct_hir_id(tcx: TyCtxt<'_>, field: &StructField) -> HirId {
    let struct_def_id = field.hir_id.owner_def_id();
    tcx.hir().as_local_hir_id(struct_def_id).unwrap()
}

/// Box field refactoring
///
/// ## Algorithm
///
/// Input
/// - `tcx: Typed context`
/// - `span: Byte range`
///
/// Steps
/// - F <- the field which should be boxed
/// - T <- the type of the struct (from Items -> StructStruct)
/// - Ps <- All StructPattern where PathInExpression has type T and F is in StructPatternFields
/// - Vs <- All StructExpressions where PathInExpression has type T and F is in StructExprFields
/// - Fs <- All FieldExpressions where Expression has type T and F is IDENTIFIER
/// - if |Ps| > 0 then abort
/// - Change F's type in T to Box<>
/// - for V in Vs:
///   - Add Box::new around V
/// - for F' in Fs
///   - Add * around F'
pub fn do_refactoring(tcx: TyCtxt, span: Span) -> Result<Vec<Change>, RefactoringError> {
    if let Some((field, index)) = collect_field(tcx, span) {
        let struct_hir_id = get_struct_hir_id(tcx, &field);

        if field.is_positional() {
            box_tuple_field::do_refactoring(tcx, struct_hir_id, index, field.ty.span)
        } else {
            box_named_field::do_refactoring(tcx, struct_hir_id, &field.ident.to_string(), field.ty.span)
        }
        
    } else {
        Err(RefactoringError::invalid_selection_with_code(
            span.lo().0,
            span.hi().0,
            &get_source(tcx, span)
        ))
    }
}
