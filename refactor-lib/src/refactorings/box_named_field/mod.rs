use rustc::ty::TyCtxt;
use rustc_hir::HirId;
use rustc_span::Span;

use crate::output_types::FileStringReplacement;
use crate::refactorings::utils::{get_source, map_change_from_span};
use crate::refactoring_invocation::RefactoringErrorInternal;
use super::visitors::{collect_local_variable_use, collect_struct_field_access_expressions};
use struct_expression_collector::collect_struct_expressions;
use struct_named_pattern_collector::collect_struct_named_patterns;
use rustc_hir::{Item, ItemKind};

mod struct_expression_collector;
pub mod struct_named_pattern_collector;

pub fn do_refactoring(tcx: TyCtxt, struct_hir_id: HirId, field_ident: &str, field_ty_span: Span) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {

    let struct_patterns = collect_struct_named_patterns(tcx, struct_hir_id, field_ident);

    if !struct_patterns.other.is_empty() {
        return Err(RefactoringErrorInternal::used_in_pattern(&field_ident));
    }
    let source_map = tcx.sess.source_map();
    
    let mut changes = vec![map_change_from_span(
        source_map,
        field_ty_span,
        format!("Box<{}>", get_source(tcx, field_ty_span)),
    )];

    let (struct_expressions, struct_expression_shorthands) = collect_struct_expressions(tcx, struct_hir_id, field_ident);

    for struct_expression in struct_expressions {
        let replacement = format!("Box::new({})", get_source(tcx, struct_expression));
        changes.push(map_change_from_span(source_map, struct_expression, replacement));
    }

    for (struct_expression, ident) in struct_expression_shorthands {
        let replacement = format!("{}: Box::new({})", ident, get_source(tcx, struct_expression));
        changes.push(map_change_from_span(source_map, struct_expression, replacement));
    }

    for field_access_expression in collect_struct_field_access_expressions(tcx, struct_hir_id, field_ident) {
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

/// Used to skip visiting derived std implementations
/// of Clone, Default, etc.
pub fn is_impl_from_std_derive_expansion(i: &Item<'_>) -> bool {
    let skip_derive_types = vec![
        "::core::clone::Clone".to_owned(),
        "::core::default::Default".to_owned(),
        "::core::fmt::Debug".to_owned(),
        "::core::cmp::Eq".to_owned(),
        "::core::hash::Hash".to_owned(),
        "::core::cmp::Ord".to_owned(),
        "::core::cmp::PartialEq".to_owned(),
        "::core::cmp::PartialOrd".to_owned(),
    ];

    if let ItemKind::Impl {
        of_trait: Some(ref t),
        ..
    } = i.kind {
        let repr: String = rustc_hir_pretty::to_string(rustc_hir_pretty::NO_ANN, |s| s.print_path(t.path, false));

        i.span.in_derive_expansion() &&
         skip_derive_types.contains(&repr)
    } else {
        false
    }
}