use super::utils::map_change_from_span;
use crate::change::Change;
use local_variable_use_collector::collect_local_variable_use;
use rustc::ty::TyCtxt;
use rustc_hir::{HirId, StructField};
use struct_def_field_collector::collect_field;
use struct_expression_collector::collect_struct_expressions;
use struct_field_access_expression_collector::collect_struct_field_access_expressions;
use struct_pattern_collector::collect_struct_patterns;
use rustc_span::Span;

use super::utils::get_source;
use crate::refactor_definition::RefactoringError;

mod local_variable_use_collector;
mod struct_def_field_collector;
mod struct_expression_collector;
mod struct_field_access_expression_collector;
mod struct_pattern_collector;

pub fn get_struct_hir_id(tcx: TyCtxt<'_>, field: &StructField) -> HirId {
    let struct_def_id = field.hir_id.owner_def_id();
    tcx.hir().as_local_hir_id(struct_def_id).unwrap()
}
pub fn get_field_ident(field: &StructField) -> String {
    format!("{}", field.ident)
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
    if let Some(field) = collect_field(tcx, span) {
        let struct_hir_id = get_struct_hir_id(tcx, &field);
        let field_ident = get_field_ident(field);
        let struct_patterns = collect_struct_patterns(tcx, struct_hir_id, field_ident.to_string());

        if !struct_patterns.other.is_empty() {
            return Err(RefactoringError::used_in_pattern(&field.ident.to_string()));
        }

        let (struct_expressions, struct_expression_shorthands) = collect_struct_expressions(tcx, struct_hir_id, field_ident.to_string());
        let field_access_expressions = collect_struct_field_access_expressions(tcx, struct_hir_id, field_ident);
        let mut changes = vec![map_change_from_span(
            tcx,
            field.ty.span,
            format!("Box<{}>", get_source(tcx, field.ty.span)),
        )];

        for struct_expression in struct_expressions {
            let replacement = format!("Box::new({})", get_source(tcx, struct_expression));
            changes.push(map_change_from_span(tcx, struct_expression, replacement));
        }

        for (struct_expression, ident) in struct_expression_shorthands {
            let replacement = format!("{}: Box::new({})", ident, get_source(tcx, struct_expression));
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
    } else {
        Err(RefactoringError::invalid_selection_with_code(
            span.lo().0,
            span.hi().0,
            &get_source(tcx, span)
        ))
    }
}
