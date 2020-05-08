use rustc_hir::{BodyId, HirId, Path};
use rustc_hir::intravisit::{NestedVisitorMap, Visitor, walk_path, walk_body};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use crate::refactoring_invocation::{QueryResult, TyContext};

struct BlockCollector<'v> {
    tcx: TyCtxt<'v>,
    pos: Vec<HirId>,
    result: Vec<Span>
}

/**
 * Given a selection (byte start, byte end) and file name, this visitor finds
 * the innermost block containing `pos`
 */
 pub fn collect_local_uses<'v>(tcx: &'v TyContext, pos: Vec<HirId>, body: BodyId) -> QueryResult<Vec<Span>> {
    let mut v = BlockCollector {
        tcx: tcx.0,
        pos,
        result: vec![]
    };

    walk_body(&mut v, tcx.0.hir().body(body));

    Ok(v.result)
}

impl<'v> Visitor<'v> for BlockCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.hir())
    }
    fn visit_path(&mut self, path: &'v Path<'v>, _id: HirId) {

        match path.res {
            rustc_hir::def::Res::Local(id) => {

                if self.pos.contains(&id) {
                    self.result.push(path.segments[0].ident.span);
                }
            },
            _ => {}
        }

        walk_path(self, path);
    }
}
