use rustc::hir::{self, intravisit};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

/**
 * Collects all function body ids in a crate
 */
struct FunctionBodyCollector<'tcx> {
    tcx: TyCtxt<'tcx>,
    body_ids: Vec<hir::BodyId>,
}

pub fn collect_function_bodies<'tcx>(tcx: TyCtxt<'tcx>) -> Vec<hir::BodyId> {
    let mut v = FunctionBodyCollector {
        tcx,
        body_ids: vec![],
    };

    intravisit::walk_crate(&mut v, tcx.hir().krate());

    v.body_ids
}

impl<'v> intravisit::Visitor<'v> for FunctionBodyCollector<'v> {
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
        self.body_ids.push(b);
        intravisit::walk_fn(self, fk, fd, b, s, id);
    }
}
