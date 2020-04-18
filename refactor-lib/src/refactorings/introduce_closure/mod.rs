use super::utils::{map_change_from_span, get_source};
use refactor_lib_types::FileStringReplacement;
use crate::refactoring_invocation::{AstDiff, QueryResult, RefactoringErrorInternal, TyContext};
use crate::refactorings::visitors::hir::{collect_cfs, collect_innermost_contained_block};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

fn get_call(tcx: TyCtxt, span: Span) -> FileStringReplacement {
    map_change_from_span(tcx.sess.source_map(), span, format!("(|| {})()", get_source(tcx, span)))
}

/// 
/// ## Algorithm
/// 
/// Input
/// - Block
/// 
/// Output
/// - A new expression containing the block as an anonymous closure
/// 
/// Preconditions
/// - Break, continue, return, `?` are not currently handled, so they must be preventet
/// 
pub fn do_refactoring(tcx: &TyContext, span: Span) -> QueryResult<AstDiff> {
    if let Some(result) = collect_innermost_contained_block(tcx.0, span) {
        // option 1: the selection is just a block
        // option 2: the selection is an assignment where the rhs is a block

        // point of closure decl: immediately before this statement

        let cf_expr = collect_cfs(tcx.0, result.0.hir_id);

        let replacements = 
        if cf_expr.has_cfs() {
            let block_src = get_source(tcx.0, span);
            let repl = cf_expr.replace_cfs(tcx.0, block_src, span.lo().0);
            let anon_inv = format!("match (|| {})() {{{}}}", repl, cf_expr.get_cf_arms());

            map_change_from_span(tcx.0.sess.source_map(), span, anon_inv)
        } else {
            get_call(tcx.0, result.0.span)
        };

        Ok(AstDiff(vec![
            replacements,
        ]))
        
    } else {
        Err(RefactoringErrorInternal::invalid_selection(span.lo().0, span.hi().0))
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils::{assert_success/*, assert_err*/};
    use quote::quote;
    // use super::RefactoringErrorInternal;
    const NAME: &str = "introduce-closure";

    #[test]
    fn introduce_closure_single_expr1() {
        assert_success(quote! {
            fn f ( ) { let _ : i32 = { 1 } ; }
        }, NAME, (11, 32),
        "fn f ( ) { let _ : i32 = (|| { 1 })() ; }");
    }
    #[test]
    fn introduce_closure_single_stmt() {
        assert_success(quote! {
            fn f ( ) { { 1 ; } }
        }, NAME, (11, 11),
        "fn f ( ) { (|| { 1 ; })() }");
    }
}