use rustc_hir::{BodyId, Expr, ExprKind, Field, FnDecl, HirId, Item};
use rustc_hir::intravisit::{FnKind, walk_expr, walk_fn, walk_item, walk_crate, NestedVisitorMap, Visitor};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use crate::refactoring_invocation::{QueryResult, RefactoringErrorInternal, TyContext};

///
/// Collect all places where a given struct occurs in a struct expression where also `field_ident` occurs.
/// 
/// # Example
/// given:
/// ```example
/// let _ = S { foo: 0 };
///             | |
///             x y
/// };
/// ```
/// then `collect_struct_expressions(S, "foo")` would return a single byte range `(x, y)`
/// 
/// but with:
/// ```example
/// let _ = S { ..bar };
/// ```
/// then `collect_struct_expressions(S, "foo")` would return an empty list
/// 
/// # Grammar
/// [Struct expression grammar](https://doc.rust-lang.org/stable/reference/expressions/struct-expr.html)
pub fn collect_struct_expressions(
    tcx: TyCtxt,
    struct_hir_id: HirId,
    field_ident: &str,
) -> QueryResult<(Vec<Span>, Vec<(Span, String)>)> {
    let mut v = StructExpressionCollector {
        tcx,
        struct_hir_id,
        field: vec![],
        shorthands: vec![],
        field_ident: field_ident.to_string(),
        body_id: None,
        err: Ok(())
    };
    
    walk_crate(&mut v, tcx.hir().krate());
    v.err?;

    Ok((v.field, v.shorthands))
}

struct StructExpressionCollector<'v> {
    tcx: TyCtxt<'v>,
    struct_hir_id: HirId,
    field: Vec<Span>,
    shorthands: Vec<(Span, String)>,
    field_ident: String,
    body_id: Option<BodyId>,
    err: QueryResult<()>
}

impl StructExpressionCollector<'_> {
    fn expr_resolves_to_struct(&mut self, expr: &Expr) -> bool {
        let body_id = if let Some(b) = self.body_id {
            b
        } else {
            self.err = Err(RefactoringErrorInternal::int("expected body id"));
            return false;
        };
        let def_id = self.tcx.hir().body_owner_def_id(body_id);
        let typecheck_table = self.tcx.typeck_tables_of(def_id);
        if let Some(expr_type) = typecheck_table.expr_ty_adjusted_opt(expr) {
            if let Some(adt_def) = expr_type.ty_adt_def() {
                return adt_def.did == self.struct_hir_id.owner.to_def_id();
            }
        } 
        false
    }
    fn handle_expr(&mut self, expr: &Expr, fields: &[Field]) {
        if self.expr_resolves_to_struct(expr) {
            for fp in fields.iter() {
                if format!("{}", fp.ident) == self.field_ident {
                    if fp.is_shorthand {
                        self.shorthands.push((fp.expr.span, fp.ident.to_string()));
                    } else {
                        self.field.push(fp.expr.span);
                    }
                }
            }
        }
    }
}

impl<'v> Visitor<'v> for StructExpressionCollector<'v> {
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
        if let ExprKind::Struct(_, fields, _) = &expr.kind {
            self.handle_expr(expr, fields);
        }
        walk_expr(self, expr);
    }
    fn visit_item(&mut self, i: &'v Item<'v>) {
        if !super::is_impl_from_std_derive_expansion(&i) {
            walk_item(self, i);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::run_ty_query;
    use crate::refactorings::visitors::collect_field;

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<(Vec<String>, Vec<String>)> + Send> {
        Box::new(move |ty| {
            let span = ty.get_span(&file_name, from, to)?;
            let (field, _) = collect_field(ty.0, span).unwrap();
            let (span1, span2) = collect_struct_expressions(ty.0, field.hir_id, &ty.get_source(span)).unwrap();

            Ok((
                span1.iter().map(|s| ty.get_source(*s)).collect::<Vec<_>>(),
                span2.iter().map(|s| ty.get_source(s.0)).collect::<Vec<_>>()))
        })
    }

    #[test]
    fn should_collect_field() {
        let input = r#"
struct S { /*START*/foo/*END*/: u32 }
fn foo() {
    let _ = S { foo: 123 };
}"#;
        let expected = Ok((
            vec!["123".to_owned()],
            vec![]));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_not_collect_when_field_doesnt_occur() {
        let input = r#"
struct S { /*START*/foo/*END*/: u32 }
fn foo() {
    let s = S { foo: 123 };
    let _ = S { ..s };
}"#;
        let expected = Ok((
            vec!["123".to_owned()],
            vec![]));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_two_occurrences() {
        let input = r#"
struct S { /*START*/foo/*END*/: u32 }
fn foo() {
    let s = S { foo: 123 };
    let _ = S { foo: 4 };
}"#;
        let expected = Ok((
            vec!["123".to_owned(), "4".to_owned()],
            vec![]));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_self() {
        let input = r#"
struct S { /*START*/foo/*END*/: u32 }
impl S {
    fn foo() {
        let _ = Self { foo: 0 };
    }
}"#;
        let expected = Ok((
            vec!["0".to_owned()],
            vec![]));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_field_init_shorthand() {
        let input = r#"
struct S { /*START*/foo/*END*/: u32 }
fn foo() {
    let foo = 0;
    S { foo };
}"#;
        let expected = Ok((
            vec![],
            vec!["foo".to_owned()]));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn shouldnt_collect_builtin_derives() {
        let input = r#"
# [derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash, Default, Debug)] 
struct S { /*START*/foo/*END*/: u32 }"#;
        let expected = Ok((
            vec![],
            vec![]));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
}
