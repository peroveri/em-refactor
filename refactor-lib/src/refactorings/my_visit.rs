use rustc::ty::TyCtxt;
// use rustc::infer::InferCtxt;
use rustc::hir::{self, intravisit as iv};

pub fn find_candidates(tcx: TyCtxt) {
    let krate = tcx.hir().krate();

    let mut visitor = MyVisitor {
        tcx
    };
    iv::walk_crate(&mut visitor, krate);
}

pub struct MyVisitor<'tcx> {
    tcx: TyCtxt<'tcx>
}

impl<'tcx> iv::Visitor<'tcx> for MyVisitor<'tcx> {
    fn nested_visit_map<'this>(&'this mut self) 
        -> iv::NestedVisitorMap<'this, 'tcx> {

        iv::NestedVisitorMap::All(&self.tcx.hir())
    }

    fn visit_item(&mut self, item: &'tcx hir::Item) {
        if let hir::ItemKind::Fn(..) = &item.node {
        }
        iv::walk_item(self, item);
    }
}