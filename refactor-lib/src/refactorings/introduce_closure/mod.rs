use super::utils::{map_change_from_span, get_source};
use crate::refactoring_invocation::{FileStringReplacement, RefactoringErrorInternal};
use crate::refactorings::visitors::hir::{collect_cfs, collect_innermost_contained_block};
use rustc::ty::TyCtxt;
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
pub fn do_refactoring(tcx: TyCtxt, span: Span) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {
    if let Some(result) = collect_innermost_contained_block(tcx, span) {
        // option 1: the selection is just a block
        // option 2: the selection is an assignment where the rhs is a block

        // point of closure decl: immediately before this statement

        let cf_expr = collect_cfs(tcx, result.0.hir_id);

        let replacements = 
        if cf_expr.has_cfs() {
            let block_src = get_source(tcx, span);
            let repl = cf_expr.replace_cfs(tcx, block_src, span.lo().0);
            let anon_inv = format!("match (|| {})() {{{}}}", repl, cf_expr.get_cf_arms());

            map_change_from_span(tcx.sess.source_map(), span, anon_inv)
        } else {
            get_call(tcx, result.0.span)
        };

        Ok(vec![
            replacements,
        ])
        
    } else {
        Err(RefactoringErrorInternal::invalid_selection(span.lo().0, span.hi().0))
    }
}

#[cfg(test)]
mod test {
    use super::test_util::{assert_success/*, assert_err*/};
    use quote::quote;
    // use super::RefactoringErrorInternal;

    #[test]
    fn introduce_closure_single_expr1() {
        assert_success(quote! {
            fn f ( ) { let _ : i32 = { 1 } ; }
        }, (11, 32),
        "fn f ( ) { let _ : i32 = (|| { 1 })() ; }");
    }
    #[test]
    fn introduce_closure_single_stmt() {
        assert_success(quote! {
            fn f ( ) { { 1 ; } }
        }, (11, 11),
        "fn f ( ) { (|| { 1 ; })() }");
    }
}
#[cfg(test)]
mod test_util {
    use super::*;
    use crate::{create_test_span, run_after_analysis};
    use crate::refactoring_invocation::get_file_content;
    pub fn assert_success(prog: quote::__rt::TokenStream, span: (u32, u32), expected: &str) {
        run_after_analysis(prog, | tcx | {
            let actual = do_refactoring(tcx, create_test_span(span.0, span.1)).unwrap();
            let res = get_file_content(&actual, tcx.sess.source_map()).unwrap();

            assert_eq!(res, expected);
        })
    }
    // pub fn assert_err(prog: quote::__rt::TokenStream, span: (u32, u32), expected: RefactoringErrorInternal) {
    //     run_after_analysis(prog, | tcx | {
    //         let actual = do_refactoring(tcx, create_test_span(span.0, span.1)).unwrap_err();

    //         assert_eq!(actual, expected);
    //     })
    // }
}