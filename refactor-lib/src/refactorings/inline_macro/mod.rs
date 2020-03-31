use rustc_span::Span;
use crate::refactoring_invocation::{AstContext, AstDiff, QueryResult, RefactoringErrorInternal};
use super::utils::map_change_from_span;
use super::visitors::collect_inline_macro;

pub fn do_refactoring(ast: &AstContext, span: Span) -> QueryResult<AstDiff>{

    let crate_ = ast.get_crate();

    if let Some((replacement, repl_span)) = collect_inline_macro(span, crate_) {
        Ok(AstDiff(vec![map_change_from_span(ast.get_source_map(), repl_span, replacement)]))
    } else {
        Err(RefactoringErrorInternal::invalid_selection(span.lo().0, span.hi().0))
    }
}
