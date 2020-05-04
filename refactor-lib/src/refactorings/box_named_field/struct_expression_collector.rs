use rustc_hir::{BodyId, Expr, ExprKind, Field, FnDecl, HirId, Item};
use rustc_hir::intravisit::{FnKind, walk_expr, walk_fn, walk_item, walk_crate, NestedVisitorMap, Visitor};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use crate::refactoring_invocation::{QueryResult, RefactoringErrorInternal};

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
    use crate::{create_test_span, run_after_analysis};
    use crate::refactorings::visitors::collect_field;
    use super::super::super::utils::get_source;
    use quote::quote;
    use quote::__private::TokenStream;

    fn create_program_match_1() -> TokenStream {
        quote! {
            struct S { foo: u32 }
            fn foo() {
                let _ = S { foo: 0 };
            }
        }
    }
    fn create_program_match_2() -> TokenStream {
        quote! {
            struct S { foo: u32 }
            fn foo() {
                let s = S { foo: 0 };
                let _ = S { ..s };
            }
        }
    }
    fn create_program_match_3() -> TokenStream {
        quote! {
            struct S { foo: u32 }
            fn foo() {
                let _ = S { foo: 0 };
                let _ = S { foo: 1 };
            }
        }
    }
    fn create_program_match_4() -> TokenStream {
        quote! {
            struct S { foo: u32 }
            impl S {
                fn foo() {
                    let _ = Self { foo: 0 };
                }
            }
        }
    }
    fn create_program_match_5() -> TokenStream {
        quote! {
            struct S { foo: u32 }
            fn bar() {
                let foo = 0;
                S { foo };
            }
        }
    }

    fn create_program_match_6() -> TokenStream {
        quote! {
            # [ derive ( Eq , PartialEq , Ord , PartialOrd , Clone , Hash , Default , Debug ) ] struct S { foo : u32 }
        }
    }
    fn get_struct_hir_id(tcx: TyCtxt<'_>) -> HirId {
        let (field, _) = collect_field(tcx, create_test_span(11, 14)).unwrap();
        let struct_def_id = field.hir_id.owner.to_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }
    fn get_struct_hir_id6(tcx: TyCtxt<'_>) -> HirId {
        let (field, _) = collect_field(tcx, create_test_span(95, 98)).unwrap();
        let struct_def_id = field.hir_id.owner.to_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }

    #[test]
    fn struct_expression_collector_should_collect_1() {
        run_after_analysis(create_program_match_1(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, "foo").unwrap();

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "0");
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_2() {
        run_after_analysis(create_program_match_2(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, "foo").unwrap();

            assert_eq!(fields.len(), 1);
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_3() {
        run_after_analysis(create_program_match_3(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, "foo").unwrap();

            assert_eq!(fields.len(), 2);
            assert_eq!(get_source(tcx, fields[0]), "0");
            assert_eq!(get_source(tcx, fields[1]), "1");
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_4() {
        run_after_analysis(create_program_match_4(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, "foo").unwrap();

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "0");
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_5() {
        run_after_analysis(create_program_match_5(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, shorthands) = collect_struct_expressions(tcx, hir_id, "foo").unwrap();

            assert_eq!(fields.len(), 0);
            assert_eq!(shorthands.len(), 1);
            assert_eq!(get_source(tcx, shorthands[0].0), "foo");
            assert_eq!(shorthands[0].1, "foo");
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_6() {
        run_after_analysis(create_program_match_6(), |tcx| {
            let hir_id = get_struct_hir_id6(tcx);
            let (fields, shorthands) = collect_struct_expressions(tcx, hir_id, "foo").unwrap();
            assert_eq!(fields.len(), 0);
            assert_eq!(shorthands.len(), 0);
            // assert_eq!("", get_source(tcx, create_test_span(0, 40)));
        });
    }
}
