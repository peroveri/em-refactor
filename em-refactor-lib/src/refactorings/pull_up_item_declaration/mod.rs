use crate::refactoring_invocation::{AstContext, AstDiff, QueryResult};
use crate::refactorings::visitors::ast::collect_innermost_block;
use em_refactor_lib_types::{create_refactor_tool_marker, defs::PULL_UP_ITEM_DECLARATIONS_STMTS}; 
use rustc_ast::ast::Stmt;
use rustc_span::Span;

/// Given a selection within a block, contiguous statements (0..n) and an expression (0|1)
/// It should pull up item declarations occuring at this block level
/// These item declarations can only be found in the selection of statements (if they are item decls.)
pub fn do_refactoring(context: &AstContext, span: Span, add_comment: bool) -> QueryResult<AstDiff> {
    let block = collect_innermost_block(context, span)?;

    let items = filter_items(&block.stmts);
    
    if contains_stmt_from_macro(&items) {
        return Err(context.source().span_err(span, true));
    }
    let items = filter_stmts_in_span(&items, span);
    let spans = items.iter().map(|s| s.span).collect::<Vec<_>>();

    let mut res = vec![];
    res.push(context.map_change(
        span.shrink_to_lo(),
        spans.iter().map(|s| context.get_source(*s)).collect::<Vec<_>>().join("")
    )?);
    for delete_span in spans {
        res.push(context.map_change(
            delete_span,
            "".to_owned(),
        )?);
    }
    if add_comment {
        res.push(context.map_change(span.shrink_to_lo(), create_refactor_tool_marker(PULL_UP_ITEM_DECLARATIONS_STMTS, false))?);
        res.push(context.map_change(span.shrink_to_hi(), create_refactor_tool_marker(PULL_UP_ITEM_DECLARATIONS_STMTS, true))?);
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
    use crate::test_utils::{run_refactoring, TestInit};
    use em_refactor_lib_types::FileStringReplacement;
    const NAME: &str = "pull-up-item-declaration";

    #[test]
    fn invalid_selection() {
        let input = r#"fn /*refactor-tool:test-id:start*/foo()/*refactor-tool:test-id:end*/ { }"#;
        let expected = Err(RefactoringErrorInternal::invalid_selection_with_code(34, 39, "foo()", false));
        
        let actual = run_refactoring(TestInit::from_refactoring(input, NAME));
        
        assert_eq!(actual, expected);
    }

    #[test]
    fn outputs_comment() {
        let input = r#"fn foo() {
    /*refactor-tool:test-id:start*/
    bar();
    fn bar() {}
    /*refactor-tool:test-id:end*/    
}"#;
        let expected = Ok(r#"fn foo() {
    /*refactor-tool:test-id:start*/fn bar() {}/*refactor-tool:pull-up-item-declaration.stmts:start*/
    bar();
    
    /*refactor-tool:pull-up-item-declaration.stmts:end*//*refactor-tool:test-id:end*/    
}"#.to_string());

        let actual = run_refactoring(TestInit::from_refactoring(input, NAME).with_add_comment());

        assert_eq!(actual, expected)
    }

    #[test]
    fn selects_from_comment() {
        let input = r#"fn foo() {
    /*refactor-tool:test-id:start*/
    bar();
    fn bar() {}
    /*refactor-tool:test-id:end*/    
}"#;
        let expected = Ok(r#"fn foo() {
    /*refactor-tool:test-id:start*/fn bar() {}
    bar();
    
    /*refactor-tool:test-id:end*/    
}"#.to_string());

        let actual = run_refactoring(TestInit::from_refactoring(input, NAME));

        assert_eq!(actual, expected)
    }

    #[test]// This test should probably be somewhere else
    #[ignore]
    fn with_changes() {
        let input = r#"fn foo() {}"#;
        let expected = Ok(r#"fn foo() {
    fn bar() {}
    bar();
}"#.to_string());
        let changes = vec![vec![
            FileStringReplacement {
                file_name: "src/main.rs".to_owned(),
                line_start: 0,
                char_start: 0,
                line_end: 0,
                char_end: 0,
                byte_start: 10,
                byte_end: 10,
                replacement: "/*refactor-tool:test-id:start*/    bar()\n    fn bar() {}/*refactor-tool:test-id:end*/".to_owned()
            }
        ]];

        let actual = run_refactoring(TestInit::from_refactoring(input, NAME).with_changes(changes));

        assert_eq!(actual, expected)
    }
}