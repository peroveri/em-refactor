use rustc::hir::{
    self,
    intravisit::{walk_crate, NestedVisitorMap, Visitor},
};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

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
    struct_hir_id: hir::HirId,
    field_ident: String,
) -> Vec<Span> {
    let mut v = StructFieldAccessExpressionCollector {
        tcx,
        struct_hir_id,
        field: vec![],
        field_ident,
        body_id: None,
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.field
}

struct StructFieldAccessExpressionCollector<'v> {
    tcx: TyCtxt<'v>,
    struct_hir_id: hir::HirId,
    field: Vec<Span>,
    field_ident: String,
    body_id: Option<hir::BodyId>,
}

impl StructFieldAccessExpressionCollector<'_> {
    fn expr_resolves_to_struct(&self, expr: &hir::Expr, hir_id: hir::HirId) -> bool {
        let typecheck_table = self.tcx.typeck_tables_of(hir_id.owner_def_id());

        let expr_ty = typecheck_table.expr_ty(expr);
        if let Some(adt_def) = expr_ty.ty_adt_def() {
            self.struct_hir_id.owner_def_id() == adt_def.did
        } else {
            false
        }
    }
}

impl<'v> Visitor<'v> for StructFieldAccessExpressionCollector<'v> {
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
        if let hir::ExprKind::Field(fexpr, ident) = &expr.kind {
            if format!("{}", ident) == self.field_ident
                && self.expr_resolves_to_struct(fexpr, expr.hir_id)
            {
                self.field.push(expr.span);
            }
        }
        hir::intravisit::walk_expr(self, expr);
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
    fn get_struct_hir_id(tcx: TyCtxt<'_>) -> hir::HirId {
        let field =
            super::super::struct_def_field_collector::collect_field(tcx, create_test_span(11, 14))
                .unwrap();
        let struct_def_id = field.hir_id.owner_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }

    #[test]
    fn struct_field_access_expression_collector_should_collect_access() {
        run_after_analysis(create_program_with_field_access(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_field_access_expressions(tcx, hir_id, "foo".to_owned());

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "s . foo");
        });
    }
}
