use rustc_hir::{Arm, Block, BodyId, ExprKind, FnDecl, HirId, MatchSource };
use rustc_hir::intravisit::{NestedVisitorMap, FnKind, walk_fn, walk_crate, Visitor, walk_expr, walk_block};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::{BytePos, Span};
use crate::refactorings::utils::get_source;

struct BlockCollector<'v> {
    tcx: TyCtxt<'v>,
    pos: Span,
    body_id: Option<BodyId>,
    selected_block: Option<&'v Block<'v>>
}

pub struct BlockInsideBlock<'v> {
    pub topmost_block: BodyId,
    // pub selected_block_id: HirId,
    pub selected_block: &'v Block<'v>
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
pub fn collect_block(tcx: TyCtxt, pos: Span) -> Option<BlockInsideBlock> {
    let mut v = BlockCollector {
        tcx,
        pos: trim_span(tcx, pos),
        body_id: None,
        selected_block: None
    };

    walk_crate(&mut v, tcx.hir().krate());

    if let (Some(a), Some(b)) = (v.body_id, v.selected_block) {
        Some(BlockInsideBlock {
            topmost_block: a,
            selected_block: b
        })
    } else {
        None
    }
}

impl BlockCollector<'_> {
    fn selection_contains_span(&self, span: Span) -> bool {
        self.pos == span.shrink_to_lo() || self.pos.contains(span)
    }
}

impl<'v> Visitor<'v> for BlockCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, Self::Map> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_fn(
        &mut self,
        fk: FnKind<'v>,
        fd: &'v FnDecl,
        b: BodyId,
        s: Span,
        id: HirId,
    ) {
        self.body_id = Some(b);
        walk_fn(self, fk, fd, b, s, id);
    }

    fn visit_block(&mut self, block: &'v Block) {
        if let Some(expr) = &block.expr {
            if let ExprKind::Match(_, ref arms, MatchSource::WhileDesugar) = (*expr).kind
            {
                if let Some(arm) = arms.first() {
                    let Arm { body, .. } = arm;
                    walk_expr(self, &**body);
                }
            }
        }
        if self.selection_contains_span(block.span) {
            self.selected_block = Some(block);
            return;
        }
        if !block.span.contains(self.pos) {
            return;
        }
        walk_block(self, block);
    }
}

#[cfg(test)]
#[allow(non_upper_case_globals)]
mod test {
    // either 
    // 1. the selection should have range of 0 and be positioned
    // to the left of a block, there should only be whitespace between 
    // the position and the block
    // 2. the selection contains a block (optionally padded by whitespace)
    use super::*;
    use crate::{create_test_span, run_after_analysis};
    use crate::refactorings::utils::get_source;
    use quote::quote;
    use quote::__rt::TokenStream;
    const block_0_lpos: fn() -> Vec<u32> = || {vec![10, 11]};
    // const block_0_rpos: fn() -> Vec<u32> = || {vec![30]};
    const block_1_lpos: fn() -> Vec<u32> = || {vec![12, 13]};
    const block_1_rpos: fn() -> Vec<u32> = || {vec![18, 19]};
    const block_2_lpos: fn() -> Vec<u32> = || {vec![20, 21]};
    const block_2_rpos: fn() -> Vec<u32> = || {vec![26, 27]};
    const program_length: u32 = 30;
    const block_0: &str = "{ { 1 } ; { 2 } ; }";
    const block_1: &str = "{ 1 }";
    const block_2: &str = "{ 2 }";
    fn invalid() -> Vec<u32> {
        let mut ret = vec![];
        let mut valid = block_0_lpos();
        valid.extend(block_1_lpos());
        valid.extend(block_2_lpos());
        for i in 0..program_length {
            if !valid.contains(&i) {
                ret.push(i);
            }
        }
        ret
    }

    fn create_program_match_1() -> TokenStream {
        quote! {
            fn foo ( ) { { 1 } ; { 2 } ; }
        }
    }

    fn assert1(tcx: TyCtxt, left: u32, right: u32, s: &str) {
        let span = create_test_span(left, right);
        let block = collect_block(tcx, span);
        assert!(block.is_some(), format!("position: ({}, {}) should result in a block. source: `{}`", left, right, get_source(tcx, span)));
        let block = block.unwrap();
        assert_eq!(get_source(tcx, block.selected_block.span), s);
    }

    #[test]
    fn block_collector_test_positions() {
        run_after_analysis(create_program_match_1(), |tcx| {
            assert_eq!(get_source(tcx, create_test_span(11, 30)), block_0);
            assert_eq!(get_source(tcx, create_test_span(13, 18)), block_1);
            assert_eq!(get_source(tcx, create_test_span(21, 26)), block_2);
        });
    }
    #[test]
    fn block_collector_should_collect_valid_position_1() {
        run_after_analysis(create_program_match_1(), |tcx| {
            for i in block_1_lpos().iter().skip(1) {
                assert1(tcx, *i, *i, block_1);
            }
        });
    }
    #[test]
    fn block_collector_should_collect_valid_position_2() {
        run_after_analysis(create_program_match_1(), |tcx| {
            for i in block_2_lpos().iter().skip(1) {
                assert1(tcx, *i, *i, block_2);
            }
        });
    }
    #[test]
    fn block_collector_should_collect_valid_selections_1() {
        run_after_analysis(create_program_match_1(), |tcx| {
            for i in block_1_lpos() {
                for j in block_1_rpos() {
                    assert1(tcx, i, j, block_1);
                }
            }
        });
    }
    #[test]
    fn block_collector_should_collect_valid_selections_2() {
        run_after_analysis(create_program_match_1(), |tcx| {
            for i in block_2_lpos() {
                for j in block_2_rpos() {
                    assert1(tcx, i, j, block_2);
                }
            }
        });
    }
    #[test]
    fn block_collector_should_not_collect_invalid_selections() {
        run_after_analysis(create_program_match_1(), |tcx| {
            for i in invalid() {
                let block = collect_block(tcx, create_test_span(i, i));
                assert!(block.is_none(), format!("position: ({}, {}) shouldn't result in a block", i, i));
            }
        });
    }
    #[test]
    fn block_collector_should_collect_1() {
        run_after_analysis(quote! {
            fn foo ( ) { { } }
        }, |tcx| {
            assert_block_collects(tcx, 13, 16, "{ }");
        });
    }
    #[test]
    fn block_collector_should_collect_2() {
        run_after_analysis(quote! {
            fn foo ( ) { if true { { } } }
        }, |tcx| {
            assert_block_collects(tcx, 23, 26, "{ }");
        });
    }
    #[test]
    fn block_collector_should_collect_3() {
        run_after_analysis(quote! {
            fn foo ( ) { while true { { } } }
        }, |tcx| {
            assert_block_collects(tcx, 23, 29, "{ }");
        });
    }
    #[test]
    fn block_collector_should_collect_4() {
        run_after_analysis(quote! {
            fn foo ( ) { for i in 0 .. 1 { { if i == 0 { } } } }
        }, |tcx| {
            assert_block_collects(tcx, 31, 48, "{ if i == 0 { } }");
        });
    }
    #[test]
    fn block_collector_should_collect_5() {
        run_after_analysis(quote! {
            fn foo ( ) { while true { { if true { } } } }
        }, |tcx| {
            assert_block_collects(tcx, 26, 41, "{ if true { } }");
        });
    }
    fn assert_block_collects(tcx: TyCtxt, from: u32, to: u32, s: &str) {
        let block = collect_block(tcx, create_test_span(from, to));
        assert!(block.is_some(), format!("position: ({}, {}) shouldn't result in a block", from, to));
        assert_eq!(s, get_source(tcx, block.unwrap().selected_block.span));
    }
    #[test]
    fn block_collector_should_collect_6() {
        run_after_analysis(quote! {
            fn foo ( ) { loop { let _ = { if true { } 1 } ; } }
        }, |tcx| {
            let block = collect_block(tcx, create_test_span(28, 45));
            assert!(block.is_some(), format!("position: ({}, {}) should result in a block", 28, 45));
        });
    }
}