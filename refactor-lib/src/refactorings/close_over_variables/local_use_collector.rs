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

#[cfg(test)]
mod test {
    use super::super::*;
    use crate::refactoring_invocation::TyContext;
    use crate::test_utils::assert_success3;
    use super::super::expr_use_visit::collect_vars;

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Vec<String>> + Send> {
        Box::new(move |ty| {
            let closure = collect_anonymous_closure(ty, ty.source().map_span(&file_name, from, to)?).unwrap();
            let vars = collect_vars(ty.0, closure.body_id)?;
            let hirs = vars.iter().map(|e| e.0).collect::<Vec<_>>();
            let spans = super::super::local_use_collector::collect_local_uses(ty, hirs, closure.body_id)?;

            let strs = spans.into_iter().map(|s| ty.get_source(s)).collect::<Vec<_>>();

            Ok(strs)
        })
    }

    #[test]
    fn should_collect_1() {
        assert_success3(
        r#"fn foo() {
    let mut i = S{f: 0};
    /*START*/(|| {
        i.f = 0;
        i.f = 0;
    })()/*END*/;
}
struct S{f: u32}"#,
            map,
            vec!["i".to_owned(), "i".to_owned()]);
    }
    // TODO: check patterns, e.g. let _ = i;
}
