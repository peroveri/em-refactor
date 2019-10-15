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
    block: Option<&'v hir::Block>,
}

pub fn collect_block(tcx: TyCtxt, pos: Span) -> Option<&hir::Block> {
    let mut v = BlockCollector {
        tcx,
        pos,
        block: None,
    };

    intravisit::walk_crate(&mut v, tcx.hir().krate());
    v.block
}

impl<'v> intravisit::Visitor<'v> for BlockCollector<'v> {
    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'v> {
        intravisit::NestedVisitorMap::All(&self.tcx.hir())
    }

    fn visit_block(&mut self, body: &'v hir::Block) {
        if let Some(expr) = &body.expr {
            if let hir::ExprKind::Match(_, ref arms, hir::MatchSource::WhileDesugar) = (*expr).kind
            {
                if let Some(arm) = arms.first() {
                    let hir::Arm { body, .. } = arm;
                    intravisit::walk_expr(self, &**body);
                }
            }
        }
        if !body.span.contains(self.pos) {
            return;
        }

        let stmts = body
            .stmts
            .iter()
            .filter(|s| self.pos.contains(s.span))
            .collect::<Vec<_>>();
        if stmts.is_empty() {
            intravisit::walk_block(self, body);
            return;
        }

        self.block = Some(body);
    }
}
