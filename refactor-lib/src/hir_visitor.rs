use rustc::hir::{intravisit};
use rustc::ty::TyCtxt;

pub struct HirVisitor<'v> {
    pub tcx: TyCtxt<'v>
}

impl<'v> intravisit::Visitor<'v> for HirVisitor<'v> {
    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'v> {
        intravisit::NestedVisitorMap::All(&self.tcx.hir())
    }
    // visit expr, etc.
}
