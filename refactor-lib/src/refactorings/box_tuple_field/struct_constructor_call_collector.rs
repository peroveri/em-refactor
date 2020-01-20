use rustc_hir::{BodyId, Expr, ExprKind, FnDecl, HirId};
use rustc_hir::intravisit::{FnKind, walk_expr, walk_fn, walk_crate, NestedVisitorMap, Visitor};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;
use if_chain::if_chain;

///
/// Collect all places where a given struct occurs in a call expression.
/// 
/// # Example
/// given:
/// ```
/// let _ = S (foo);
///            | |
///            x y
/// ```
/// then `collect_struct_constructor_calls(S, "0")` would return a single byte range `(x, y)`
/// 
/// # Grammar
/// ```
/// ```
/// [Struct expression grammar](https://doc.rust-lang.org/stable/reference/expressions/struct-expr.html)
pub fn collect_struct_constructor_calls(
    tcx: TyCtxt,
    struct_hir_id: HirId,
    field_index: usize,
) -> Vec<Span> {
    let mut v = StructConstructorCallCollector {
        tcx,
        struct_hir_id,
        field: vec![],
        field_index,
        body_id: None,
    };
    
    walk_crate(&mut v, tcx.hir().krate());

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
        let typecheck_table = self.tcx.typeck_tables_of(expr.hir_id.owner_def_id());
        if let Some(expr_type) = typecheck_table.expr_ty_adjusted_opt(expr) {
            if let Some(adt_def) = expr_type.ty_adt_def() {
                return adt_def.did == self.struct_hir_id.owner_def_id();
            }
        } 
        false
    }
    fn handle_call(&mut self, expr: &Expr, function: &Expr, args: &[Expr]) {
        if let ExprKind::Path(qpath) = &function.kind {
            let typecheck_table = self.tcx.typeck_tables_of(expr.hir_id.owner_def_id());

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
        if let ExprKind::Call(function, args) = &expr.kind {
            self.handle_call(expr, function, args);
        }
        walk_expr(self, expr);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{create_test_span, run_after_analysis};
    use crate::refactorings::visitors::collect_field;
    use super::super::super::utils::get_source;
    use quote::quote;

    fn create_program_match_6() -> quote::__rt::TokenStream {
        quote! {
            struct S ( i32 );
            fn foo() {
                let _ = S(123);
            }
        }
    }
    fn create_program_match_7() -> quote::__rt::TokenStream {
        quote! {
            struct S ( i32 );
            fn foo(s: S, b: bool) -> S {
                if b {
                    foo(s, false)
                } else {
                    s
                }
            }
        }
    }

    fn get_struct_tuple_hir_id(tcx: TyCtxt<'_>) -> HirId {
        let (field, _) = collect_field(tcx, create_test_span(11, 14)).unwrap();
        let struct_def_id = field.hir_id.owner_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }

    #[test]
    fn struct_expression_collector_should_collect_6() {
        run_after_analysis(create_program_match_6(), |tcx| {
            let hir_id = get_struct_tuple_hir_id(tcx);
            let fields = collect_struct_constructor_calls(tcx, hir_id, 0);

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "123");
        });
    }
    #[test]
    fn struct_expression_collector_should_not_collect_7() {
        run_after_analysis(create_program_match_7(), |tcx| {
            let hir_id = get_struct_tuple_hir_id(tcx);
            let fields = collect_struct_constructor_calls(tcx, hir_id, 0);

            assert_eq!(fields.len(), 0);
        });
    }
}
