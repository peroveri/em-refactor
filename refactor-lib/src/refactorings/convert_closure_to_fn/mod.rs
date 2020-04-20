use crate::refactoring_invocation::{AstDiff, QueryResult, TyContext};
use rustc_span::Span;
use crate::refactorings::visitors::hir::{collect_anonymous_closure};

// fn fresh_name() -> String {"foo".to_owned()}

/// Convert anonymous closure to function
/// 
/// ## Algorithm
/// 1. For each parameter
/// 2. If return value:
pub fn do_refactoring(tcx: &TyContext, span: Span) -> QueryResult<AstDiff> {
    let closure = collect_anonymous_closure(tcx.0, span)?;

    let mut changes = vec![];

    changes.push(tcx.map_change(closure.body_span.with_hi(closure.body_span.lo()), "".to_owned()));

    Ok(AstDiff(changes))
}
