use rustc_hir::{BodyId, Expr, ExprKind, FnDecl, HirId};
use rustc_hir::intravisit::{FnKind, walk_expr, walk_fn, walk_crate, NestedVisitorMap, Visitor};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;

///
/// Collect all places where a given struct occurs in a field access expression where the field is `field_ident`.
///
/// # Example
/// given:
/// ```
/// let s = S {foo: 0};
/// let _ = s.foo;
///         |   |
///         x   y
/// ```
/// then `collect_field_access_expressions(S, "foo")` would return a single byte range `(x, y)`
///
/// # Syntax
/// ```
/// FieldExpression :
///   Expression . IDENTIFIER
/// ```
/// [Field access expressions grammar](https://doc.rust-lang.org/stable/reference/expressions/field-expr.html)
pub fn collect_struct_field_access_expressions(
    tcx: TyCtxt,
    struct_hir_id: HirId,
    field_ident: &str,
) -> Vec<Span> {
    let mut v = StructFieldAccessExpressionCollector {
        tcx,
        struct_hir_id,
        field: vec![],
        field_ident: field_ident.to_string(),
        body_id: None,
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.field
}

struct StructFieldAccessExpressionCollector<'v> {
    tcx: TyCtxt<'v>,
    struct_hir_id: HirId,
    field: Vec<Span>,
    field_ident: String,
    body_id: Option<BodyId>,
}

impl StructFieldAccessExpressionCollector<'_> {
    fn expr_resolves_to_struct(&self, expr: &Expr, hir_id: HirId) -> bool {
        let typecheck_table = self.tcx.typeck_tables_of(hir_id.owner_def_id());

        let expr_ty = typecheck_table.expr_ty_adjusted(expr);

        if let Some(adt_def) = expr_ty.ty_adt_def() {
            self.struct_hir_id.owner_def_id() == adt_def.did
        } else {
            false
        }
    }
}

impl<'v> Visitor<'v> for StructFieldAccessExpressionCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, Self::Map> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_fn(
        &mut self,
        fk: FnKind<'v>,
        fd: &'v FnDecl,
        body_id: BodyId,
        s: Span,
        h: HirId,
    ) {
        self.body_id = Some(body_id);
        walk_fn(self, fk, fd, body_id, s, h);
    }
    fn visit_expr(&mut self, expr: &'v Expr) {
        if let ExprKind::Field(fexpr, ident) = &expr.kind {
            if format!("{}", ident) == self.field_ident
                && self.expr_resolves_to_struct(fexpr, expr.hir_id)
            {
                self.field.push(expr.span);
            }
        }
        walk_expr(self, expr);
    }
}

#[cfg(test)]
mod test {
    use super::super::super::utils::get_source;
    use super::*;
    use crate::{create_test_span, run_after_analysis};
    use quote::quote;

    fn create_program_with_field_access() -> quote::__rt::TokenStream {
        quote! {
            struct S { foo: u32 }
            fn foo() {
                let s = S {foo: 0};
                let _ = s.foo;
            }
        }
    }
    fn create_program_with_field_access_self_param() -> quote::__rt::TokenStream {
        quote! {
            struct S { foo: u32 }
            impl S {
                fn foo(&self) {
                    let _ = self.foo;
                }
            }
        }
    }
    fn create_program_with_field_access_self_type() -> quote::__rt::TokenStream {
        quote! {
            struct S { foo: u32 }
            impl S {
                fn foo(s: Self) {
                    let _ = s.foo;
                }
            }
        }
    }
    fn get_struct_hir_id(tcx: TyCtxt<'_>) -> HirId {
        let (field, _) =
            super::super::struct_def_field_collector::collect_field(tcx, create_test_span(11, 14))
                .unwrap();
        let struct_def_id = field.hir_id.owner_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }
    
    #[test]
    fn struct_field_access_expression_collector_should_collect_access() {
        run_after_analysis(create_program_with_field_access(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_field_access_expressions(tcx, hir_id, "foo");

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "s . foo");
        });
    }
    #[test]
    fn struct_field_access_expression_collector_should_collect_access_self_param() {
        run_after_analysis(create_program_with_field_access_self_param(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_field_access_expressions(tcx, hir_id, "foo");

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "self . foo");
        });
    }
    #[test]
    fn struct_field_access_expression_collector_should_collect_access_self_type() {
        run_after_analysis(create_program_with_field_access_self_type(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_field_access_expressions(tcx, hir_id, "foo");

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "s . foo");
        });
    }
}
