use rustc_hir::intravisit::{walk_crate, NestedVisitorMap, Visitor, walk_expr};
use rustc_hir::{Expr, ExprKind, HirId, QPath};
use rustc_hir::def::Res;
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

///
/// Collects all uses of a local variable
/// 
/// # Example
/// Given:
/// ```example
/// let foo = 0;
///     |
///     hir_id: x
/// bar(foo);
///     | |
///     x0x1
/// let _ = foo;
///         | |
///         y0y1
/// ```
/// then `collect_local_variable_use(x)` will return `(x0, x1)` and `(y0, y1)`
///
pub fn collect_local_variable_use(tcx: TyCtxt, hir_id: HirId) -> Vec<Span> {
    let mut v = LocalVariableUseCollector {
        tcx,
        hir_id,
        uses: vec![],
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.uses
}

struct LocalVariableUseCollector<'v> {
    tcx: TyCtxt<'v>,
    hir_id: HirId,
    uses: Vec<Span>,
}

impl<'v> Visitor<'v> for LocalVariableUseCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.hir())
    }
    fn visit_expr(&mut self, expr: &'v Expr) {
        if expr.hir_id == self.hir_id {
            self.uses.push(expr.span);
        }
        if let ExprKind::Path(qpath) = &expr.kind {
            if let QPath::Resolved(_, path) = qpath {
                if let Res::Local(hir_id) = path.res {
                    if hir_id == self.hir_id {
                        self.uses.push(expr.span);
                    }
                }
            }
        }

        walk_expr(self, expr);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::run_ty_query;
    use crate::refactorings::visitors::collect_field;
    use crate::refactoring_invocation::{QueryResult, TyContext};
    use crate::refactorings::utils::get_struct_hir_id;
    use crate::refactorings::box_named_field::struct_named_pattern_collector::collect_struct_named_patterns;

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Vec<String>> + Send> {
        Box::new(move |ty| {
            let span = ty.source().map_span(&file_name, from, to)?;

            let (field, _) = collect_field(ty.0, span).unwrap();
            let struct_hir_id = get_struct_hir_id(ty.0, &field);
            let patterns = collect_struct_named_patterns(ty, struct_hir_id, &ty.get_source(span)).new_bindings;

            let mut ret = vec![];
            for id in patterns {
                ret.extend(collect_local_variable_use(ty.0, id));
            }
            Ok(ret.iter().map(|s| ty.get_source(*s)).collect::<Vec<_>>())
        })
    }

    #[test]
    fn should_collect_uses_1() {
        let input = r#"
            struct S { /*START*/field/*END*/: u32 }
            fn foo() {
                match (S {field: 0}) {
                    S {field} => {
                        let i = &field;
                    }
                }
            }"#;
        let expected = Ok(
            vec!["field".to_owned()]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
}
