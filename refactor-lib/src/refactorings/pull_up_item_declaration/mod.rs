use crate::refactoring_invocation::{AstContext, AstDiff, QueryResult};
use crate::refactorings::visitors::ast::collect_innermost_block;
use rustc_ast::ast::Stmt;
use rustc_span::{BytePos, Span};

/// Given a selection within a block, contiguous statements (0..n) and an expression (0|1)
/// It should pull up item declarations occuring at this block level
/// These item declarations can only be found in the selection of statements (if they are item decls.)
pub fn do_refactoring(context: &AstContext, span: Span) -> QueryResult<AstDiff> {
    let block = collect_innermost_block(context, span)?;

    let items = filter_items(&block.stmts);
    
    if contains_stmt_from_macro(&items) {
        return Err(context.span_err(span));
    }
    let items = filter_stmts_in_span(&items, span);
    let spans = items.iter().map(|s| s.span).collect::<Vec<_>>();

    let mut res = vec![];
    res.push(context.map_change(
        span.with_lo(span.lo() - BytePos(0)).shrink_to_lo(),
        spans.iter().map(|s| context.get_source(*s)).collect::<Vec<_>>().join("")
    ));
    for delete_span in spans {
        res.push(context.map_change(
            delete_span,
            "".to_owned(),
        ));
    }
    Ok(AstDiff(res))
}

fn filter_items(stmts: &[Stmt]) -> Vec<&Stmt> {
    stmts.iter().filter(|s| s.is_item()).collect::<Vec<_>>() 
}
fn contains_stmt_from_macro(stmts: &[&Stmt]) -> bool {
    stmts.iter().any(|s| s.span.from_expansion())
}
fn filter_stmts_in_span<'a>(stmts: &[&'a Stmt], span: Span) -> Vec<&'a Stmt> {
    stmts.iter().filter(|s| span.contains(s.span)).map(|s| *s).collect::<Vec<_>>()
}

#[cfg(test)]
mod test {
    use crate::refactoring_invocation::RefactoringErrorInternal;
    use crate::test_utils::{assert_success, assert_err, run_refactoring, TestInit};
    use quote::quote;
    const NAME: &str = "pull-up-item-declaration";
    #[test]
    fn pull_up_item_declaration_fn_decl() {
        assert_success(quote! {
            fn f ( ) { 0 ; fn g ( ) { } g ( ) ; }
        }, NAME, (10, 36),  
        "fn f ( ) {fn g ( ) { } 0 ;  g ( ) ; }");
    }
    #[test]
    fn pull_up_item_declaration_2_fn_decl() {
        assert_success(quote! {
            fn f ( ) { 0 ; fn g ( ) { } fn h ( ) { } g ( ) ; }
        }, NAME, (10, 49),
        "fn f ( ) {fn g ( ) { }fn h ( ) { } 0 ;   g ( ) ; }");
    }
    #[test]
    fn pull_up_item_declaration_no_items() {
        assert_success(quote! {
            fn f ( ) { 0 ; 1 ; }
        }, NAME, (10, 19),
        "fn f ( ) { 0 ; 1 ; }");
    }
    #[test]
    fn pull_up_item_declaration_macro_inv() {
        assert_success(quote! {
            fn f ( ) { print ! ( "{}" , 1 ) ; fn g ( ) { } print ! ( "{}" , 2 ) ; }
        }, NAME, (10, 70),
        r#"fn f ( ) {fn g ( ) { } print ! ( "{}" , 1 ) ;  print ! ( "{}" , 2 ) ; }"#);
    }
    #[test]
    fn pull_up_item_declaration_macro_declaring_item() {
        assert_err(quote! {
            macro_rules ! foo { ( ) => { fn bar ( ) { } } } fn f ( ) { foo ! ( ) ; }
        }, NAME, (58, 71),  
        RefactoringErrorInternal::invalid_selection_with_code(58, 71, " foo ! ( ) ; "));
    }
    #[test]
    fn pull_up_item_declaration_invalid_selection() {
        assert_err(quote! {
            fn f ( ) { 0 ; 1 ; }
        }, NAME, (0, 4),  
        RefactoringErrorInternal::invalid_selection_with_code(0, 4, "fn f"));
    }
    #[test]
    fn selects_from_comment() {
        let expected = Ok(r#"fn foo() {
    /*refactor-tool:test-id:start*/fn bar() {}
    bar();
    
    /*refactor-tool:test-id:end*/    
}"#.to_string());

        let actual = run_refactoring(TestInit::from_refactoring(
r#"fn foo() {
    /*refactor-tool:test-id:start*/
    bar();
    fn bar() {}
    /*refactor-tool:test-id:end*/    
}"#, NAME));

        assert_eq!(actual, expected)
    }
}