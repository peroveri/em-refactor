use rustc_hir::{Arm, BodyId, Block, FnDecl, HirId, ExprKind, MatchSource};
use rustc_hir::intravisit::{NestedVisitorMap, Visitor, FnKind, walk_fn, walk_expr, walk_block, walk_crate};
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
    result: Option<(&'v Block<'v>, BodyId)>
}

pub fn collect_block(tcx: TyCtxt, pos: Span) -> Option<(&Block, BodyId)> {
    let mut v = BlockCollector {
        tcx,
        pos,
        body_id: None,
        result: None
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.result
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

    fn visit_block(&mut self, body: &'v Block) {
        if let Some(expr) = &body.expr {
            if let ExprKind::Match(_, ref arms, MatchSource::WhileDesugar) = (*expr).kind
            {
                if let Some(arm) = arms.first() {
                    let Arm { body, .. } = arm;
                    walk_expr(self, &**body);
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
            walk_block(self, body);
            return;
        }

        self.result = Some((body, self.body_id.unwrap()));
    }
}
