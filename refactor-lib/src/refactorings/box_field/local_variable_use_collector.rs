use rustc::hir::{
    self,
    intravisit::{self, walk_crate, NestedVisitorMap, Visitor},
};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

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
pub fn collect_local_variable_use(tcx: TyCtxt, hir_id: hir::HirId) -> Vec<Span> {
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
    hir_id: hir::HirId,
    uses: Vec<Span>,
}

impl<'v> Visitor<'v> for LocalVariableUseCollector<'v> {
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, 'v> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_expr(&mut self, expr: &'v hir::Expr) {
        if expr.hir_id == self.hir_id {
            self.uses.push(expr.span);
        }
        if let hir::ExprKind::Path(qpath) = &expr.kind {
            if let hir::QPath::Resolved(_, path) = qpath {
                if let hir::def::Res::Local(hir_id) = path.res {
                    if hir_id == self.hir_id {
                        self.uses.push(expr.span);
                    }
                }
            }
        }

        intravisit::walk_expr(self, expr);
    }
}

#[cfg(test)]
mod test {
    use super::super::{
        collect_field, collect_struct_patterns, get_field_ident, get_source, get_struct_hir_id,
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

    fn init_test(tcx: TyCtxt<'_>) -> hir::HirId {
        let field = collect_field(tcx, create_test_span(11, 16));
        assert!(field.is_some());
        let field = field.unwrap();
        let struct_hir_id = get_struct_hir_id(tcx, &field);
        let patterns = collect_struct_patterns(tcx, struct_hir_id, get_field_ident(field));

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
