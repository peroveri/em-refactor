use rustc_hir::intravisit::{walk_crate, NestedVisitorMap, Visitor, walk_expr};
use rustc_hir::{Expr, ExprKind, HirId, QPath};
use rustc_hir::def::Res;
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;

///
/// Collects all uses of a local variable
/// 
/// # Example
/// Given:
/// ```
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
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, Self::Map> {
        NestedVisitorMap::All(&self.tcx.hir())
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
    use super::super::{
        collect_field, collect_struct_named_patterns, get_source, get_struct_hir_id, StructFieldType
    };
    use super::*;
    use crate::{create_test_span, run_after_analysis};
    use quote::quote;

    fn create_program_1() -> quote::__rt::TokenStream {
        quote! {
            struct S {field: u32}
            fn foo() {
                match (S {field: 0}) {
                    S {field} => {
                        let i = &field;
                    }
                }
            }
        }
    }

    fn init_test(tcx: TyCtxt<'_>) -> HirId {
        let field = collect_field(tcx, create_test_span(11, 16));
        assert!(field.is_some());
        let (field, _) = field.unwrap();
        let struct_hir_id = get_struct_hir_id(tcx, &field);
        let patterns = collect_struct_named_patterns(tcx, struct_hir_id, StructFieldType::from(field, 0));

        assert_eq!(1, patterns.new_bindings.len());
        patterns.new_bindings[0]
    }

    #[test]
    fn local_variable_use_collector_should_collect_uses_1() {
        run_after_analysis(create_program_1(), |tcx| {
            let uses = collect_local_variable_use(tcx, init_test(tcx));

            assert_eq!(1, uses.len());
            assert_eq!("field", get_source(tcx, uses[0]));
        });
    }
}
