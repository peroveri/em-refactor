use rustc_hir::HirId;
use rustc_span::Span;

use crate::refactoring_invocation::{AstDiff, QueryResult, RefactoringErrorInternal, TyContext};
use super::utils::{get_source, get_struct_hir_id};
use super::{box_named_field, box_tuple_field};
use super::visitors::collect_field;

pub struct StructPatternCollection {
    pub new_bindings: Vec<HirId>,
    pub other: Vec<Span>
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

pub fn do_refactoring(tcx: &TyContext, span: Span, _add_comment: bool) -> QueryResult<AstDiff> {
    if let Some((field, index)) = collect_field(tcx.0, span) {
        let struct_hir_id = get_struct_hir_id(tcx.0, &field);

        if field.is_positional() {
            Ok(AstDiff(box_tuple_field::do_refactoring(tcx.0, struct_hir_id, index, field.ty.span)?))
        } else {
            box_named_field::do_refactoring(tcx.0, struct_hir_id, &field.ident.to_string(), field.ty.span)
        }
        
    } else {
        Err(RefactoringErrorInternal::invalid_selection_with_code(
            span.lo().0,
            span.hi().0,
            &get_source(tcx.0, span)
        ))
    }
}
