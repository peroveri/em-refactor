use rustc_ast::ast::{Block, NodeId};
use rustc_ast::visit::{FnKind, Visitor, walk_block, walk_crate, walk_fn};
use rustc_span::Span;
use crate::refactoring_invocation::{AstContext, QueryResult, RefactoringErrorInternal};

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
        Err(RefactoringErrorInternal::invalid_selection_with_code(
            span.lo().0,
            span.hi().0,
            &context.get_source(span)
        ))
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
    use crate::refactoring_invocation::{AfterExpansion, SourceCodeRange};
    use crate::test_utils::{assert_err2, assert_success2};
    use quote::quote;

    struct InnermostBlockCollector(SourceCodeRange);

    impl AfterExpansion for InnermostBlockCollector {
        fn after_expansion(&self, ast: &AstContext) -> QueryResult<String> {
            collect_innermost_block(ast, ast.map_range_to_span(&self.0)?)
                .map(|b| ast.get_source(b.span))
        }
    }
    fn map(from: u32, to: u32) -> Box<dyn Fn(String) -> Box<dyn AfterExpansion + Send>> { 
        Box::new(move |name| Box::new(InnermostBlockCollector(SourceCodeRange {
            file_name: name,
            from,
            to
        })))
    }
    #[test]
    fn block_collector_fn_with_single_block() {
        assert_success2(quote! {
            fn foo ( ) { 1 ; 2 ; }
        }, map(12, 21), "{ 1 ; 2 ; }");
    }
    #[test]
    fn block_collector_should_collect_outermost() {
        assert_success2(quote! {
            fn foo ( ) { { 1 ; 2 ; } }
        }, map(12, 25), "{ { 1 ; 2 ; } }");
    }
    #[test]
    fn block_collector_should_collect_innermost() {
        assert_success2(quote! {
            fn foo ( ) { { 1 ; 2 ; } }
        }, map(14, 23), "{ 1 ; 2 ; }");
    }
    #[test]
    fn block_collector_shouldnt_collect_const() {
        assert_err2(quote! {
            const _ : i32 = { 1 } ;
        }, map(17, 20), RefactoringErrorInternal::invalid_selection_with_code(17, 20, " 1 "));
    }
    #[test]
    fn block_collector_shouldnt_collect() {
        assert_err2(quote! {
            fn f ( ) { }
        }, map(0, 12), RefactoringErrorInternal::invalid_selection_with_code(0, 12, "fn f ( ) { }"));
    }
}