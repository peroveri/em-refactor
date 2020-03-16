use rustc_ast::ast::{Block, Crate};
use rustc_ast::visit::{Visitor, walk_block, walk_crate};
use rustc_span::Span;

struct BlockCollector<'v> {
    span: Span,
    result: Option<&'v Block>
}

/**
 * Given a selection (byte start, byte end) and file name, this visitor finds
 * the innermost block containing `pos`
 */
pub fn collect_innermost_block<'v>(crate_: &Crate, span: Span) -> Option<&Block> {
    let mut v = BlockCollector {
        span,
        result: None
    };

    walk_crate(&mut v, crate_);

    v.result
}

impl<'v> Visitor<'v> for BlockCollector<'v> {
    fn visit_block(&mut self, block: &'v Block) {
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
    use super::test_util::assert_success;
    use quote::quote;

    #[test]
    fn block_collector_fn_with_single_block() {
        assert_success(quote! {
            fn foo ( ) { 1 ; 2 ; }
        }, (12, 21), "{ 1 ; 2 ; }");
    }
    #[test]
    fn block_collector_should_collect_outermost() {
        assert_success(quote! {
            fn foo ( ) { { 1 ; 2 ; } }
        }, (12, 25), "{ { 1 ; 2 ; } }");
    }
    #[test]
    fn block_collector_should_collect_innermost() {
        assert_success(quote! {
            fn foo ( ) { { 1 ; 2 ; } }
        }, (14, 23), "{ 1 ; 2 ; }");
    }
}

#[cfg(test)]
mod test_util {
    use super::*;
    use quote::__rt::TokenStream;
    use crate::{create_test_span, run_after_expansion};
    use crate::refactorings::utils::get_source_from_compiler;

    pub fn assert_success(prog: TokenStream, span: (u32, u32), expected: &str) {
        run_after_expansion(prog, |queries, c| {
            let (crate_, ..) = 
            &*queries
                .expansion()
                .unwrap()
                .peek_mut();
        
            let block = collect_innermost_block(crate_, create_test_span(span.0, span.1)).unwrap();
            
            assert_eq!(get_source_from_compiler(c, block.span), expected);
        });
    }
}