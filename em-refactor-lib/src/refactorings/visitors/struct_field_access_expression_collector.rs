use rustc_hir::{BodyId, Expr, ExprKind, FnDecl, HirId};
use rustc_hir::intravisit::{FnKind, walk_expr, walk_fn, walk_crate, NestedVisitorMap, Visitor};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use crate::refactoring_invocation::TyContext;

///
/// Collect all places where a given struct occurs in a field access expression where the field is `field_ident`.
///
/// # Example
/// given:
/// ```example
/// let s = S {foo: 0};
/// let _ = s.foo;
///         |   |
///         x   y
/// ```
/// then `collect_field_access_expressions(S, "foo")` would return a single byte range `(x, y)`
///
/// # Syntax
/// ```example
/// FieldExpression :
///   Expression . IDENTIFIER
/// ```
/// [Field access expressions grammar](https://doc.rust-lang.org/stable/reference/expressions/field-expr.html)
pub fn collect_struct_field_access_expressions(
    tcx: &TyContext,
    struct_hir_id: HirId,
    field_ident: &str,
) -> Vec<Span> {
    let mut v = StructFieldAccessExpressionCollector {
        tcx: tcx.0,
        struct_hir_id,
        field: vec![],
        field_ident: field_ident.to_string(),
        body_id: None,
    };

    walk_crate(&mut v, tcx.0.hir().krate());

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
        let typecheck_table = self.tcx.typeck_tables_of(hir_id.owner.to_def_id());

        let expr_ty = typecheck_table.expr_ty_adjusted(expr);

        if let Some(adt_def) = expr_ty.ty_adt_def() {
            self.struct_hir_id.owner.to_def_id() == adt_def.did
        } else {
            false
        }
    }
}

impl<'v> Visitor<'v> for StructFieldAccessExpressionCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.hir())
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
    use super::*;
    use crate::test_utils::run_ty_query;
    use crate::refactoring_invocation::QueryResult;
    use super::super::collect_field;

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Vec<String>> + Send> {
        Box::new(move |ty| {
            let span = ty.source().map_span(&file_name, from, to)?;
            let (field, _) = collect_field(ty.0, span).unwrap();
            let hir_id = ty.get_struct_hir_id(field);
            let xs = collect_struct_field_access_expressions(ty, hir_id, &field.ident.as_str().to_string());

            Ok(xs.iter().map(|s| ty.get_source(*s)).collect::<Vec<_>>())
        })
    }
    #[test]
    fn should_collect_access() {
        let input = r#"
            struct S { /*START*/foo/*END*/: u32 }
            fn foo() {
                let s = S {foo: 0};
                let _ = s.foo;
            }"#;

        let expected = Ok(vec!["s.foo".to_owned()]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_access_self_param() {
        let input = r#"
            struct S { /*START*/foo/*END*/: u32 }
            impl S {
                fn foo(&self) {
                    let _ = self.foo;
                }
            }"#;

        let expected = Ok(vec!["self.foo".to_owned()]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_access_self_type() {
        let input = r#"
            struct S { /*START*/foo/*END*/: u32 }
            impl S {
                fn foo(s: Self) {
                    let _ = s.foo;
                }
            }"#;

        let expected = Ok(vec!["s.foo".to_owned()]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
}
