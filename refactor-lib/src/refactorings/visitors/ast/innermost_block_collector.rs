use rustc_ast::ast::{Block, NodeId};
use rustc_ast::visit::{FnKind, Visitor, walk_block, walk_crate, walk_fn};
use rustc_span::Span;
use crate::refactoring_invocation::{AstContext, QueryResult};

struct BlockVisitorCollector<'v> {
    span: Span,
    result: Option<&'v Block>,
    in_fn: i32
}

/**
 * Given a selection (byte start, byte end) and file name, this visitor finds
 * the innermost block containing `pos`
 */
pub fn collect_innermost_block<'v>(context: &'v AstContext, span: Span) -> QueryResult<&'v Block> {
    let mut v = BlockVisitorCollector {
        span,
        result: None,
        in_fn: 0
    };

    walk_crate(&mut v, &context.get_crate());
    if let Some(r) = v.result {
        Ok(r)
    } else {
        Err(context.source().span_err(span, false))
    }
}

impl<'v> Visitor<'v> for BlockVisitorCollector<'v> {
    fn visit_fn(&mut self, fk: FnKind<'v>, s: Span, _: NodeId) {
        if s.contains(self.span) {
            self.in_fn += 1;
            walk_fn(self, fk, s);
            self.in_fn -= 1;
        }
    }
    fn visit_block(&mut self, block: &'v Block) {
        if self.in_fn <= 0 {
            return;
        }
        walk_block(self, block);
        if self.result.is_some() {
            return;
        }
        if block.span.contains(self.span) {
            self.result = Some(block);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{run_ast_query, TestContext};
    use crate::refactoring_invocation::RefactoringErrorInternal;

    fn map(ctx: TestContext) -> Box<dyn Fn(&AstContext) -> QueryResult<String> + Send> { 
        Box::new(move |ast| {
            let span = ast.source().map_span(&ctx.main_path, ctx.selection.unwrap().0, ctx.selection.unwrap().1)?;
            collect_innermost_block(ast, span)
                .map(|block| ast.get_source(block.span))
        })
    }
    #[test]
    fn fn_with_single_block() {
        let input = r#"
        fn foo () {/*START*/
            1; 2;
        /*END*/}"#;
        let expected = Ok(r#"{/*START*/
            1; 2;
        /*END*/}"#.to_owned());

        let actual = run_ast_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_outermost() {
        let input = r#"
        fn foo () {/*START*/
            { 1 ; 2 ; }
        /*END*/}"#;
        let expected = Ok(r#"{/*START*/
            { 1 ; 2 ; }
        /*END*/}"#.to_owned());

        let actual = run_ast_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_innermost() {
        let input = r#"
        fn foo () {
            {/*START*/ 1 ; 2 ; /*END*/}
        }"#;
        let expected = Ok(
            "{/*START*/ 1 ; 2 ; /*END*/}".to_owned());

        let actual = run_ast_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn shouldnt_collect_const() {
        let input = "const _: i32 = {/*START*/ 1 /*END*/};";
        let expected = Err(RefactoringErrorInternal::invalid_selection_with_code(25, 28, " 1 ", false));

        let actual = run_ast_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn shouldnt_collect() {
        let input = "/*START*/fn f () { }/*END*/";
        let expected = Err(RefactoringErrorInternal::invalid_selection_with_code(9, 20, "fn f () { }", false));

        let actual = run_ast_query(input, map);

        assert_eq!(actual, expected);
    }
}