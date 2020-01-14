use rustc_hir::{BodyId, Expr, ExprKind, FnDecl, HirId};
use rustc_hir::intravisit::{FnKind, walk_expr, walk_fn, walk_crate, NestedVisitorMap, Visitor};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;

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
    struct_hir_id: HirId,
    field_ident: String,
) -> (Vec<Span>, Vec<(Span, String)>) {
    let mut v = StructPatternCollector {
        tcx,
        struct_hir_id,
        field: vec![],
        shorthands: vec![],
        field_ident,
        body_id: None,
    };

    walk_crate(&mut v, tcx.hir().krate());

    (v.field, v.shorthands)
}

struct StructPatternCollector<'v> {
    tcx: TyCtxt<'v>,
    struct_hir_id: HirId,
    field: Vec<Span>,
    shorthands: Vec<(Span, String)>,
    field_ident: String,
    body_id: Option<BodyId>,
}

impl StructPatternCollector<'_> {
    fn path_resolves_to_struct(&self, expr: &Expr) -> bool {
        let typecheck_table = self.tcx.typeck_tables_of(expr.hir_id.owner_def_id());
        if let Some(expr_type) = typecheck_table.expr_ty_adjusted_opt(expr) {
            if let Some(adt_def) = expr_type.ty_adt_def() {
                return adt_def.did == self.struct_hir_id.owner_def_id();
            }
        } 
        false
    }
}

impl<'v> Visitor<'v> for StructPatternCollector<'v> {
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
        if let ExprKind::Struct(_, fields, _) = &expr.kind {
            if self.path_resolves_to_struct(expr) {
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
        walk_expr(self, expr);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{create_test_span, run_after_analysis};
    use super::super::super::utils::get_source;
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
                let _ = S { foo: 1 };
            }
        }
    }
    fn create_program_match_4() -> quote::__rt::TokenStream {
        quote! {
            struct S { foo: u32 }
            impl S {
                fn foo() {
                    let _ = Self { foo: 0 };
                }
            }
        }
    }
    fn create_program_match_5() -> quote::__rt::TokenStream {
        quote! {
            struct S { foo: u32 }
            fn bar() {
                let foo = 0;
                S { foo };
            }
        }
    }
    fn get_struct_hir_id(tcx: TyCtxt<'_>) -> HirId {
        let field =
            super::super::struct_def_field_collector::collect_field(tcx, create_test_span(11, 14))
                .unwrap();
        let struct_def_id = field.hir_id.owner_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }

    #[test]
    fn struct_expression_collector_should_collect_1() {
        run_after_analysis(create_program_match_1(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, "foo".to_owned());

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "0");
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_2() {
        run_after_analysis(create_program_match_2(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, "foo".to_owned());

            assert_eq!(fields.len(), 1);
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_3() {
        run_after_analysis(create_program_match_3(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, "foo".to_owned());

            assert_eq!(fields.len(), 2);
            assert_eq!(get_source(tcx, fields[0]), "0");
            assert_eq!(get_source(tcx, fields[1]), "1");
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_4() {
        run_after_analysis(create_program_match_4(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, "foo".to_owned());

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "0");
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_5() {
        run_after_analysis(create_program_match_5(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, shorthands) = collect_struct_expressions(tcx, hir_id, "foo".to_owned());

            assert_eq!(fields.len(), 0);
            assert_eq!(shorthands.len(), 1);
            assert_eq!(get_source(tcx, shorthands[0].0), "foo");
            assert_eq!(shorthands[0].1, "foo");
        });
    }
}
