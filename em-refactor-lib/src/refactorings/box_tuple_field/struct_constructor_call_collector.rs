use rustc_hir::{BodyId, Expr, ExprKind, FnDecl, HirId};
use rustc_hir::intravisit::{FnKind, walk_expr, walk_fn, walk_crate, NestedVisitorMap, Visitor};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use if_chain::if_chain;
use crate::refactoring_invocation::TyContext;

///
/// Collect all places where a given struct occurs in a call expression.
/// 
/// # Example
/// given:
/// ```example
/// let _ = S (foo);
///            | |
///            x y
/// ```
/// then `collect_struct_constructor_calls(S, "0")` would return a single byte range `(x, y)`
/// 
/// # Grammar
/// [Struct expression grammar](https://doc.rust-lang.org/stable/reference/expressions/struct-expr.html)
pub fn collect_struct_constructor_calls(
    tcx: &TyContext,
    struct_hir_id: HirId,
    field_index: usize,
) -> Vec<Span> {
    let mut v = StructConstructorCallCollector {
        tcx: tcx.0,
        struct_hir_id,
        field: vec![],
        field_index,
        body_id: None,
    };
    
    walk_crate(&mut v, tcx.0.hir().krate());

    v.field
}

struct StructConstructorCallCollector<'v> {
    tcx: TyCtxt<'v>,
    struct_hir_id: HirId,
    field: Vec<Span>,
    field_index: usize,
    body_id: Option<BodyId>,
}

impl StructConstructorCallCollector<'_> {
    fn expr_resolves_to_struct(&self, expr: &Expr) -> bool {
        let typecheck_table = self.tcx.typeck_tables_of(expr.hir_id.owner.to_def_id());
        if let Some(expr_type) = typecheck_table.expr_ty_adjusted_opt(expr) {
            if let Some(adt_def) = expr_type.ty_adt_def() {
                return adt_def.did == self.struct_hir_id.owner.to_def_id();
            }
        } 
        false
    }
    fn handle_call(&mut self, expr: &Expr, function: &Expr, args: &[Expr]) {
        if let ExprKind::Path(qpath) = &function.kind {
            let typecheck_table = self.tcx.typeck_tables_of(expr.hir_id.owner.to_def_id());

            if_chain! {
                if let Some(defid) = typecheck_table.qpath_res(qpath, function.hir_id).opt_def_id();
                if self.tcx.is_constructor(defid);
                if self.expr_resolves_to_struct(expr);
                if let Some(expr_init) = args.get(self.field_index);
                then {
                    self.field.push(expr_init.span);
                }
            }
        }
    }
}

impl<'v> Visitor<'v> for StructConstructorCallCollector<'v> {
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
        if let ExprKind::Call(function, args) = &expr.kind {
            self.handle_call(expr, function, args);
        }
        walk_expr(self, expr);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::run_ty_query;
    use crate::refactoring_invocation::QueryResult;
    use crate::refactorings::visitors::collect_field;
    use crate::refactorings::utils::get_struct_hir_id;

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Vec<String>> + Send> {
        Box::new(move |ty| {
            let span = ty.source().map_span(&file_name, from, to)?;

            let (field, _) = collect_field(ty.0, span).unwrap();
            let hir_id = get_struct_hir_id(ty.0, field);
            let fields = collect_struct_constructor_calls(ty, hir_id, 0);

            Ok(fields.iter().map(|s| ty.get_source(*s)).collect::<Vec<_>>())
        })
    }

    #[test]
    fn should_collect_6() {
        let input = r#"
            struct S (/*START*/i32/*END*/);
            fn foo() {
                let _ = S(123);
            }"#;

        let expected = Ok(vec!["123".to_owned()]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_not_collect_7() {
        let input = r#"
            struct S (/*START*/i32/*END*/);
            fn foo(s: S, b: bool) -> S {
                if b {
                    foo(s, false)
                } else {
                    s
                }
            }"#;

        let expected = Ok(vec![]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
}
