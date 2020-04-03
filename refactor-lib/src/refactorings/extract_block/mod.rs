use super::visitors::hir::collect_innermost_block;
use crate::refactoring_invocation::{AstDiff, QueryResult, TyContext};
use rustc_span::Span;

mod expr_use_visit;
mod push_stmt_into_block;
mod variable_use_collection;

/// Extract block
/// 
/// ## Algorithm
/// 
/// Steps
/// Block <- The block (innermost) containing A;B;C
/// A <- Statements before B
/// B <- Statements to be extracted
/// C <- Statements after B
/// 
/// If B ends with an expression:
///    Add { around B } and return 
/// End
/// 
/// Vs <- Locals declared in B and used in C
/// 
/// 
/// for each stmt
/// how should it be moved?
/// a. identical (cut & paste)
/// b. add declaration and assign at start of block + add var in expression at end of block
pub fn do_refactoring(tcx: &TyContext, span: Span) -> QueryResult<AstDiff> {
    let selection = collect_innermost_block(tcx, span)?;

    let vars = push_stmt_into_block::collect_variables_overlapping_span(tcx, selection.1, span);
    let statements_source = tcx.get_source(span);

    // Add declaration with assignment, and expression at end of block
    // for variables declared in the selection and used later
    let (let_b, expr, end) = vars.get_let_expr_end();
    let new_block_source = format!("{}{{{}{}}}{}", let_b, statements_source, expr, end);

    Ok(AstDiff(vec![tcx.map_change(
        span,
        new_block_source,
    )]))
}

#[cfg(test)]
mod test {
    use crate::test_utils::{assert_success/*, assert_err*/};
    use quote::quote;
    const NAME: &str = "extract-block";

    #[test]
    fn extract_block_single_expr() {
        assert_success(quote! {
            fn f ( ) -> i32 { 0 }
        }, NAME,  (17, 20),
        r#"fn f ( ) -> i32 {{ 0 }}"#);
    }
    #[test]
    fn extract_block_single_stmt() {
        assert_success(quote! {
            fn f ( ) { 0 ; }
        }, NAME, (10, 15),
        r#"fn f ( ) {{ 0 ; }}"#);
    }
}