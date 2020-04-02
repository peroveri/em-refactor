use super::visitors::hir::collect_innermost_block;
use crate::refactoring_invocation::{AstDiff, QueryResult, TyContext};
use rustc_hir::BodyId;
use rustc_span::Span;

mod expr_use_visit;
mod push_stmt_into_block;
mod variable_use_collection;

fn extract_block(
    tcx: &TyContext,
    body_id: BodyId,
    span: Span,
) -> String {
    let vars = push_stmt_into_block::collect_variables_overlapping_span(tcx.0, body_id, span);
    let source = tcx.get_source(span);

    // Add declaration with assignment, and expression at end of block
    // for variables declared in the selection and used later
    let (let_b, expr, end) = vars.get_let_expr_end();
    format!("{}{{{}{}}}{}", let_b, source, expr, end)
}


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

    Ok(AstDiff(vec![tcx.map_change(
        span,
        extract_block(tcx, selection.1, span),
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