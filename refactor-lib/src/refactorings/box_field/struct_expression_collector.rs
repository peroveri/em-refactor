use rustc::hir::{
    self,
    intravisit::{walk_crate, NestedVisitorMap, Visitor},
};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

///
/// Collect all places where a given struct occurs in a struct expression where also `field_ident` occurs.
/// 
/// # Example
/// given:
/// ```
/// let _ = S { foo: 0 };
///             | |
///             x y
/// };
/// ```
/// then `collect_struct_expressions(S, "foo")` would return a single byte range `(x, y)`
/// 
/// but with:
/// ```
/// let _ = S { ..bar };
/// ```
/// then `collect_struct_expressions(S, "foo")` would return an empty list
/// 
/// # Grammar
/// ```
/// ```
/// [Struct expression grammar](https://doc.rust-lang.org/stable/reference/expressions/struct-expr.html)
pub fn collect_struct_expressions(
    tcx: TyCtxt,
    struct_hir_id: hir::HirId,
    field_ident: String,
) -> Vec<Span> {
    let mut v = StructPatternCollector {
        tcx,
        struct_hir_id,
        field: vec![],
        field_ident,
        body_id: None,
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.field
}

struct StructPatternCollector<'v> {
    tcx: TyCtxt<'v>,
    struct_hir_id: hir::HirId,
    field: Vec<Span>,
    field_ident: String,
    body_id: Option<hir::BodyId>,
}

impl StructPatternCollector<'_> {
    fn path_resolves_to_struct(&self, qpath: &hir::QPath, h: hir::HirId) -> bool {
        let typecheck_table = self.tcx.typeck_tables_of(h.owner_def_id());

        if let hir::QPath::Resolved(Some(ty), _) = qpath {
            let qp_type = typecheck_table.node_type(ty.hir_id);
            let struct_type = typecheck_table.node_type(self.struct_hir_id);
            rustc::ty::TyS::same_type(struct_type, qp_type)
        } else {
            let res = typecheck_table.qpath_res(qpath, h);
            let res_def_id = res.def_id();
            res_def_id == self.struct_hir_id.owner_def_id()
        }
    }
}

impl<'v> Visitor<'v> for StructPatternCollector<'v> {
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, 'v> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_fn(
        &mut self,
        fk: hir::intravisit::FnKind<'v>,
        fd: &'v hir::FnDecl,
        body_id: hir::BodyId,
        s: Span,
        h: hir::HirId,
    ) {
        self.body_id = Some(body_id);
        hir::intravisit::walk_fn(self, fk, fd, body_id, s, h);
    }
    fn visit_expr(&mut self, expr: &'v hir::Expr) {
        if let hir::ExprKind::Struct(qpath, fields, _) = &expr.kind {
            if self.path_resolves_to_struct(qpath, expr.hir_id) {
                for fp in fields {
                    if format!("{}", fp.ident) == self.field_ident {
                        self.field.push(fp.expr.span);
                    }
                }
            }
        }
        hir::intravisit::walk_expr(self, expr);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{create_test_span, run_test};
    use quote::quote;

    fn create_program_match_1() -> quote::__rt::TokenStream {
        quote! {
            struct S { foo: u32 }
            fn foo() {
                let _ = S { foo: 0 };
            }
        }
    }
    fn create_program_match_2() -> quote::__rt::TokenStream {
        quote! {
            struct S { foo: u32 }
            fn foo() {
                let s = S { foo: 0 };
                let _ = S { ..s };
            }
        }
    }
    fn create_program_match_3() -> quote::__rt::TokenStream {
        quote! {
            struct S { foo: u32 }
            fn foo() {
                let _ = S { foo: 0 };
                let _ = S { foo: 0 };
            }
        }
    }
    fn get_struct_hir_id(tcx: TyCtxt<'_>) -> hir::HirId {
        let field =
            super::super::struct_def_field_collector::collect_field(tcx, create_test_span(11, 14))
                .unwrap();
        let struct_def_id = field.hir_id.owner_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }

    #[test]
    fn struct_expression_collector_should_collect_1() {
        run_test(create_program_match_1(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_expressions(tcx, hir_id, "foo".to_owned());

            assert_eq!(fields.len(), 1);
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_2() {
        run_test(create_program_match_2(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_expressions(tcx, hir_id, "foo".to_owned());

            assert_eq!(fields.len(), 1);
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_3() {
        run_test(create_program_match_3(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_expressions(tcx, hir_id, "foo".to_owned());

            assert_eq!(fields.len(), 2);
        });
    }
}
