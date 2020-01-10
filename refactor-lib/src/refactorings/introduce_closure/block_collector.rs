use rustc_hir::{Arm, Block, BodyId, ExprKind, FnDecl, HirId, MatchSource };
use rustc_hir::intravisit::{NestedVisitorMap, FnKind, walk_fn, walk_crate, Visitor, walk_expr, walk_block};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;

/**
 * Given a selection (byte start, byte end) and file name, this visitor finds
 * the innermost block containing `pos`
 */
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

pub fn collect_block(tcx: TyCtxt, pos: Span) -> Option<BlockInsideBlock> {
    let mut v = BlockCollector {
        tcx,
        pos,
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
        if self.pos.contains(block.span) {
            self.selected_block = Some(block);
        }
        if !block.span.contains(self.pos) {
            return;
        }
        walk_block(self, block);
    }
}
