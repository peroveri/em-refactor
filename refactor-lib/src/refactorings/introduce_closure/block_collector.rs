use rustc::hir::{self, intravisit};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

/**
 * Given a selection (byte start, byte end) and file name, this visitor finds
 * the innermost block containing `pos`
 */
struct BlockCollector<'v> {
    tcx: TyCtxt<'v>,
    pos: Span,
    body_id: Option<hir::BodyId>,
    selected_block: Option<&'v hir::Block>
}

pub struct BlockInsideBlock<'v> {
    pub topmost_block: hir::BodyId,
    // pub selected_block_id: hir::HirId,
    pub selected_block: &'v hir::Block
}

pub fn collect_block(tcx: TyCtxt, pos: Span) -> Option<BlockInsideBlock> {
    let mut v = BlockCollector {
        tcx,
        pos,
        body_id: None,
        selected_block: None
    };

    intravisit::walk_crate(&mut v, tcx.hir().krate());

    if let (Some(a), Some(b)) = (v.body_id, v.selected_block) {
        Some(BlockInsideBlock {
            topmost_block: a,
            selected_block: b
        })
    } else {
        None
    }
}

impl<'v> intravisit::Visitor<'v> for BlockCollector<'v> {
    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'v> {
        intravisit::NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_fn(
        &mut self,
        fk: intravisit::FnKind<'v>,
        fd: &'v hir::FnDecl,
        b: hir::BodyId,
        s: Span,
        id: hir::HirId,
    ) {
        self.body_id = Some(b);
        intravisit::walk_fn(self, fk, fd, b, s, id);
    }

    fn visit_block(&mut self, block: &'v hir::Block) {
        if let Some(expr) = &block.expr {
            if let hir::ExprKind::Match(_, ref arms, hir::MatchSource::WhileDesugar) = (*expr).kind
            {
                if let Some(arm) = arms.first() {
                    let hir::Arm { body, .. } = arm;
                    intravisit::walk_expr(self, &**body);
                }
            }
        }
        if self.pos.contains(block.span) {
            self.selected_block = Some(block);
        }
        if !block.span.contains(self.pos) {
            return;
        }
        intravisit::walk_block(self, block);
    }
}
