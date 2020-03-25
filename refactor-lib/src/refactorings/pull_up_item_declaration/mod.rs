use crate::refactoring_invocation::{AstContext, AstDiff, QueryResult, AfterExpansionRefactoring, RefactoringErrorInternal, SourceCodeRange};
use crate::refactorings::visitors::ast::collect_innermost_block;
use rustc_span::{BytePos, Span};

pub struct PullUpItemDeclRefa(pub SourceCodeRange);

impl AfterExpansionRefactoring for PullUpItemDeclRefa {
    fn refactor(&self, ast: &AstContext) -> QueryResult<AstDiff> {
        do_refactoring(ast, ast.map_range_to_span(&self.0)?)
    }
}

/// Given a selection within a block, contiguous statements (0..n) and an expression (0|1)
/// It should pull up item declarations occuring at this block level
/// These item declarations can only be found in the selection of statements (if they are item decls.)
fn do_refactoring(context: &AstContext, span: Span) -> QueryResult<AstDiff> {
    // TODO: the steps should be clear from the body here
    // Find innermost block B
    // For each statement in B (not nested), that is item decl
    //   if it comes from macro exp, error
    // Delete and insert those statements at top
    let item_declarations = collect_item_declarations(context, span)?
        .into_iter().filter(|s| span.contains(*s)).collect::<Vec<_>>();
    
    let mut res = vec![];
    res.push(context.map_change(
        span.with_lo(span.lo() - BytePos(0)).shrink_to_lo(),
        item_declarations.iter().map(|s| context.get_source(*s)).collect::<Vec<_>>().join("")
    ));
    for delete in item_declarations {
        res.push(context.map_change(
            delete,
            "".to_owned(),
        ));
    }
    Ok(AstDiff(res))
}

fn collect_item_declarations(context: &AstContext, span: Span) -> Result<Vec<Span>, RefactoringErrorInternal> {
    
    let block = collect_innermost_block(context, span)?;

    let items = block.stmts.iter()
        .filter(|s| s.is_item())
        .map(|s| s.span)
        .collect::<Vec<_>>();

    if items.iter().any(|s| s.from_expansion()) {
        return Err(RefactoringErrorInternal::invalid_selection_with_code(
            span.lo().0,
            span.hi().0,
            "contains macro returning item"
        ));
    }

    Ok(items.into_iter().filter(|s| span.contains(*s)).collect::<Vec<_>>())
}

#[cfg(test)]
mod test {
    use crate::refactoring_invocation::RefactoringErrorInternal;
    use crate::test_utils::{assert_success, assert_err};
    use quote::quote;
    const REFACTORING_NAME: &str = "pull-up-item-declaration";
    #[test]
    fn pull_up_item_declaration_fn_decl() {
        assert_success(quote! {
            fn f ( ) { 0 ; fn g ( ) { } g ( ) ; }
        }, REFACTORING_NAME, (10, 36),  
        "fn f ( ) {fn g ( ) { } 0 ;  g ( ) ; }");
    }
    #[test]
    fn pull_up_item_declaration_2_fn_decl() {
        assert_success(quote! {
            fn f ( ) { 0 ; fn g ( ) { } fn h ( ) { } g ( ) ; }
        }, REFACTORING_NAME, (10, 49),
        "fn f ( ) {fn g ( ) { }fn h ( ) { } 0 ;   g ( ) ; }");
    }
    #[test]
    fn pull_up_item_declaration_no_items() {
        assert_success(quote! {
            fn f ( ) { 0 ; 1 ; }
        }, REFACTORING_NAME, (10, 19),
        "fn f ( ) { 0 ; 1 ; }");
    }
    #[test]
    fn pull_up_item_declaration_macro_inv() {
        assert_success(quote! {
            fn f ( ) { print ! ( "{}" , 1 ) ; fn g ( ) { } print ! ( "{}" , 2 ) ; }
        }, REFACTORING_NAME, (10, 70),
        r#"fn f ( ) {fn g ( ) { } print ! ( "{}" , 1 ) ;  print ! ( "{}" , 2 ) ; }"#);
    }
    #[test]
    fn pull_up_item_declaration_macro_declaring_item() {
        assert_err(quote! {
            macro_rules ! foo { ( ) => { fn bar ( ) { } } } fn f ( ) { foo ! ( ) ; }
        }, REFACTORING_NAME, (58, 71),  
        RefactoringErrorInternal::invalid_selection_with_code(58, 71, "contains macro returning item"));
    }
    #[test]
    fn pull_up_item_declaration_invalid_selection() {
        assert_err(quote! {
            fn f ( ) { 0 ; 1 ; }
        }, REFACTORING_NAME, (0, 4),  
        RefactoringErrorInternal::invalid_selection_with_code(0, 4, "fn f"));
    }
}