use rustc_hir::{Block, BodyId, FnDecl, HirId };
use rustc_hir::intravisit::{NestedVisitorMap, FnKind, walk_block, walk_fn, walk_crate, Visitor};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::{BytePos, Span};
use crate::refactorings::utils::get_source;
use super::walk_desugars;

struct BlockCollector<'v> {
    tcx: TyCtxt<'v>,
    pos: Span,
    body_ids: Vec<BodyId>,
    selected_block: Option<(&'v Block<'v>, BodyId)>
}

fn trim_span(tcx: TyCtxt, mut span: Span) -> Span {
    let source = get_source(tcx, span);
    if let Some(d) = source.find(|c| !char::is_whitespace(c)) {
        span = span.with_lo(BytePos(span.lo().0 + d as u32));
    }
    if let Some(d) = source.rfind(|c| !char::is_whitespace(c)) {
        let diff = source.len() - d - 1;
        span = span.with_hi(BytePos(span.hi().0 - diff as u32));
    }
    span
}

/**
 * Span should either contain an assignment expression where the right hand side is a block expression
 * or a single block expression.
 * The block expression should not be the body of a function, loop, etc.
 */
pub fn collect_innermost_contained_block<'v>(tcx: TyCtxt<'v>, pos: Span) -> Option<(&'v Block<'v>, BodyId)> {
    let mut v = BlockCollector {
        tcx,
        pos: trim_span(tcx, pos),
        body_ids: vec![],
        selected_block: None
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.selected_block
}

impl BlockCollector<'_> {
    fn selection_contains_span(&self, span: Span) -> bool {
        self.pos == span.shrink_to_lo() || self.pos.contains(span)
    }
}

impl<'v> Visitor<'v> for BlockCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.hir())
    }
    fn visit_fn(
        &mut self,
        fk: FnKind<'v>,
        fd: &'v FnDecl,
        b: BodyId,
        s: Span,
        id: HirId,
    ) {
        self.body_ids.push(b);
        walk_fn(self, fk, fd, b, s, id);
        self.body_ids.pop();
    }

    fn visit_block(&mut self, block: &'v Block) {
        if let Some(expr) = &block.expr {
            walk_desugars(self, &expr.kind);
        }
        if self.selection_contains_span(block.span) {
            self.selected_block = Some((block, *self.body_ids.last().unwrap()));
            return;
        }
        if !block.span.contains(self.pos) {
            return;
        }
        walk_block(self, block);
    }
}

#[cfg(test)]
mod test {
    use super::test_utils::{/*assert_fail,*/ assert_success};
    use quote::quote;
    
    #[test]
    fn collect_innermost_contained_block_1() {
        let p = quote! {
            fn f ( ) { { } }
        };
        assert_success(p.clone(), (11, 11), "{ }");
        assert_success(p.clone(), (10, 15), "{ }");
        // TODO: check parent in AST instead of HIR?
        // assert_fail(p.clone(), (9, 16));
        // assert_fail(p.clone(), (9, 9));
    }
    #[test]
    fn collect_innermost_contained_block_should_collect_while() {
        let p = quote! {
            fn f ( ) { while { true } { { 1 ; } } }
        };
        // assert_fail(p.clone(), (9, 39));
        // assert_fail(p.clone(), (26, 37));
        assert_success(p.clone(), (17, 25), "{ true }");
        assert_success(p.clone(), (28, 35), "{ 1 ; }");
    }
}

#[cfg(test)]
mod test_utils {
    use super::*;
    use quote::__rt::TokenStream;
    use crate::{create_test_span, run_after_analysis};
    use crate::refactorings::utils::get_source;

    pub fn assert_success(prog: TokenStream, span: (u32, u32), expected: &str) {
        run_after_analysis(prog, |tcx| {
            let (block, _) = collect_innermost_contained_block(tcx, create_test_span(span.0, span.1)).unwrap();
            
            assert_eq!(get_source(tcx, block.span), expected);
        });
    }
    // pub fn assert_fail(prog: TokenStream, span: (u32, u32)) {
    //     run_after_analysis(prog, |tcx| {
    //         assert!(collect_innermost_contained_block(tcx, create_test_span(span.0, span.1)).is_none());
    //     });
    // }
}