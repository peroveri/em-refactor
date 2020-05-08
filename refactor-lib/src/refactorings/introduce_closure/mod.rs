use super::utils::{map_change_from_span, get_source};
use refactor_lib_types::{create_refactor_tool_marker, FileStringReplacement, defs::INTRODUCE_CLOSURE_CALL_EXPR};
use crate::refactoring_invocation::{AstDiff, QueryResult, TyContext};
use crate::refactorings::visitors::hir::{collect_cfs, collect_innermost_contained_block};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

fn get_call(tcx: TyCtxt, span: Span, add_comment: bool) -> QueryResult<FileStringReplacement> {
    map_change_from_span(tcx.sess.source_map(), span, format!("{}(|| {})(){}", get_start_comment(add_comment), get_source(tcx, span), get_end_comment(add_comment)))
}

fn get_end_comment(add_comment: bool) -> String {
    if add_comment {
        create_refactor_tool_marker(INTRODUCE_CLOSURE_CALL_EXPR, true)
    } else {
        "".to_owned()
    }
}
fn get_start_comment(add_comment: bool) -> String {
    if add_comment {
        create_refactor_tool_marker(INTRODUCE_CLOSURE_CALL_EXPR, false)
    } else {
        "".to_owned()
    }
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
pub fn do_refactoring(tcx: &TyContext, span: Span, add_comment: bool) -> QueryResult<AstDiff> {
    if let Some(result) = collect_innermost_contained_block(tcx.0, span) {
        // option 1: the selection is just a block
        // option 2: the selection is an assignment where the rhs is a block

        // point of closure decl: immediately before this statement

        let cf_expr = collect_cfs(tcx.0, result.0.hir_id);

        let replacements = 
        if cf_expr.has_cfs() {
            let block_src = get_source(tcx.0, span);
            let repl = cf_expr.replace_cfs(tcx.0, block_src, span.lo().0);
            let anon_inv = format!("match {}(|| {})(){} {{{}}}", get_start_comment(add_comment), repl, get_end_comment(add_comment), cf_expr.get_cf_arms());

            map_change_from_span(tcx.0.sess.source_map(), span, anon_inv)?
        } else {
            get_call(tcx.0, result.0.span, add_comment)?
        };

        Ok(AstDiff(vec![
            replacements,
        ]))
        
    } else {
        Err(tcx.source().span_err(span))
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils::{run_refactoring, TestInit};
    const NAME: &str = "introduce-closure";

    #[test]
    fn introduce_closure_single_expr1() {
        let input = r#"fn foo() {
    /*refactor-tool:test-id:start*/let _ : i32 = { 1 };/*refactor-tool:test-id:end*/   
}"#;
        let expected = Ok(r#"fn foo() {
    /*refactor-tool:test-id:start*/let _ : i32 = (|| { 1 })();/*refactor-tool:test-id:end*/   
}"#.to_owned());

        let actual = run_refactoring(TestInit::from_refactoring(input, NAME));
        assert_eq!(actual, expected);
    }

    #[test]
    fn adds_comment_1() {
        let input = r#"fn foo() {
    /*refactor-tool:test-id:start*/let _ : i32 = { 1 };/*refactor-tool:test-id:end*/   
}"#;
        let expected = Ok(r#"fn foo() {
    /*refactor-tool:test-id:start*/let _ : i32 = /*refactor-tool:introduce-closure.call-expr:start*/(|| { 1 })()/*refactor-tool:introduce-closure.call-expr:end*/;/*refactor-tool:test-id:end*/   
}"#.to_owned());

        let actual = run_refactoring(
            TestInit::from_refactoring(input, NAME)
            .with_add_comment());
        assert_eq!(actual, expected);
    }
    
    #[test]
    fn adds_comment_2() {
        let input = r#"fn foo() {
    loop {
        /*refactor-tool:test-id:start*/{
            break;
            5
        }/*refactor-tool:test-id:end*/;
    }
}"#;
        let expected = Ok(r#"fn foo() {
    loop {
        /*refactor-tool:test-id:start*/match /*refactor-tool:introduce-closure.call-expr:start*/(|| {
            return (1, None);
            (0, Some(5))
        })()/*refactor-tool:introduce-closure.call-expr:end*/ {
(1, _) => break,
(_, a) => a.unwrap()}/*refactor-tool:test-id:end*/;
    }
}"#.to_owned());

        let actual = run_refactoring(
            TestInit::from_refactoring(input, NAME)
            .with_add_comment());
        assert_eq!(actual, expected);
    }
    // TODO: test add_comment for call expression and match-expression
}