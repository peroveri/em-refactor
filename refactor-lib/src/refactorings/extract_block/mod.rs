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
pub fn do_refactoring(tcx: &TyContext, span: Span, add_comment: bool) -> QueryResult<AstDiff> {
    let selection = collect_innermost_block(tcx, span)?;

    let vars = push_stmt_into_block::collect_variables_overlapping_span(tcx, selection.1, span)?;
    let statements_source = tcx.get_source(span);

    let (block_start, block_end) = (get_block_start(add_comment), get_block_end(add_comment));
    // Add declaration with assignment, and expression at end of block
    // for variables declared in the selection and used later
    let new_block_source = match vars.len() {
        0 => format!("{}{}{}", block_start, statements_source, block_end),
        1 => format!("let {} = \n{}{}{}{};", vars.decls_fmt(), block_start, statements_source, vars.idents_fmt(), block_end),
        _ => format!("let ({}) = \n{}{}({}){};", vars.decls_fmt(), block_start, statements_source, vars.idents_fmt(), block_end)
    };

    Ok(AstDiff(vec![tcx.map_change(
        span,
        new_block_source,
    )]))
}

fn get_block_start(add_comment: bool) -> String {
    if add_comment {
        "/*refactor-tool:extract-block.block:start*/{"
    } else {
        "{"
    }.to_owned()
}
fn get_block_end(add_comment: bool) -> String {
    if add_comment {
        "}/*refactor-tool:extract-block.block:end*/"
    } else {
        "}"
    }.to_owned()
}

#[cfg(test)]
mod test {
    use crate::test_utils::{run_refactoring, TestInit};
    const NAME: &str = "extract-block";

    #[test]
    fn outputs_from_comment() {
        let input = r#"fn foo() {
    /*refactor-tool:test-id:start*/let i = 0;/*refactor-tool:test-id:end*/   
}"#;
        let expected = Ok(r#"fn foo() {
    /*refactor-tool:test-id:start*//*refactor-tool:extract-block.block:start*/{let i = 0;}/*refactor-tool:extract-block.block:end*//*refactor-tool:test-id:end*/   
}"#.to_owned());

        let actual = run_refactoring(TestInit::from_refactoring(input, NAME).with_add_comment());
        assert_eq!(actual, expected);
    }
    #[test]
    fn selects_from_comment() {
        let input = r#"fn foo() {
    /*refactor-tool:test-id:start*/let i = 0;/*refactor-tool:test-id:end*/   
}"#;
        let expected = Ok(r#"fn foo() {
    /*refactor-tool:test-id:start*/{let i = 0;}/*refactor-tool:test-id:end*/   
}"#.to_owned());

        let actual = run_refactoring(TestInit::from_refactoring(input, NAME));
        assert_eq!(actual, expected);
    }
}