use rustc_hir::{BodyId, Expr, ExprKind, Field, FnDecl, HirId};
use rustc_hir::intravisit::{FnKind, walk_expr, walk_fn, walk_crate, NestedVisitorMap, Visitor};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;
use super::StructFieldType;
use if_chain::if_chain;

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
    field_ident: StructFieldType,
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
    field_ident: StructFieldType,
    body_id: Option<BodyId>,
}

impl StructPatternCollector<'_> {
    fn expr_resolves_to_struct(&self, expr: &Expr) -> bool {
        let typecheck_table = self.tcx.typeck_tables_of(expr.hir_id.owner_def_id());
        if let Some(expr_type) = typecheck_table.expr_ty_adjusted_opt(expr) {
            if let Some(adt_def) = expr_type.ty_adt_def() {
                return adt_def.did == self.struct_hir_id.owner_def_id();
            }
        } 
        false
    }
    fn handle_expr(&mut self, expr: &Expr, fields: &[Field], field_name: &str) {
        if self.expr_resolves_to_struct(expr) {
            for fp in fields.iter() {
                if format!("{}", fp.ident) == field_name {
                    if fp.is_shorthand {
                        self.shorthands.push((fp.expr.span, fp.ident.to_string()));
                    } else {
                        self.field.push(fp.expr.span);
                    }
                }
            }
        }
    }
    fn handle_call(&mut self, expr: &Expr, function: &Expr, args: &[Expr], index: usize) {
        if let ExprKind::Path(qpath) = &function.kind {
            let typecheck_table = self.tcx.typeck_tables_of(expr.hir_id.owner_def_id());

            if_chain! {
                if let Some(defid) = typecheck_table.qpath_res(qpath, function.hir_id).opt_def_id();
                if self.tcx.is_constructor(defid);
                if self.expr_resolves_to_struct(expr);
                if let Some(expr_init) = args.get(index);
                then {
                    self.field.push(expr_init.span);
                }
            }
        }
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
            if let StructFieldType::Named(name) = &self.field_ident {
                let name = name.to_string();
                self.handle_expr(expr, fields, &name);
            }
        } else if let ExprKind::Call(function, args) = &expr.kind {
            if let StructFieldType::Tuple(index) = self.field_ident {
                self.handle_call(expr, function, args, index);
            }
        }
        walk_expr(self, expr);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{create_test_span, run_after_analysis};
    use super::super::struct_def_field_collector::collect_field;
    use super::super::super::utils::get_source;
    use quote::quote;

    fn tup(i: usize) -> StructFieldType {
        StructFieldType::Tuple(i)
    }
    fn field(s: &str) -> StructFieldType {
        StructFieldType::Named(s.to_string())
    }

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

    fn get_struct_hir_id(tcx: TyCtxt<'_>) -> HirId {
        let (field, _) = collect_field(tcx, create_test_span(11, 14)).unwrap();
        let struct_def_id = field.hir_id.owner_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }
    fn get_struct_tuple_hir_id(tcx: TyCtxt<'_>) -> HirId {
        let (field, _) = collect_field(tcx, create_test_span(11, 14)).unwrap();
        let struct_def_id = field.hir_id.owner_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }

    #[test]
    fn struct_expression_collector_should_collect_1() {
        run_after_analysis(create_program_match_1(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, field("foo"));

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "0");
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_2() {
        run_after_analysis(create_program_match_2(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, field("foo"));

            assert_eq!(fields.len(), 1);
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_3() {
        run_after_analysis(create_program_match_3(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, field("foo"));

            assert_eq!(fields.len(), 2);
            assert_eq!(get_source(tcx, fields[0]), "0");
            assert_eq!(get_source(tcx, fields[1]), "1");
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_4() {
        run_after_analysis(create_program_match_4(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, _) = collect_struct_expressions(tcx, hir_id, field("foo"));

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "0");
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_5() {
        run_after_analysis(create_program_match_5(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let (fields, shorthands) = collect_struct_expressions(tcx, hir_id, field("foo"));

            assert_eq!(fields.len(), 0);
            assert_eq!(shorthands.len(), 1);
            assert_eq!(get_source(tcx, shorthands[0].0), "foo");
            assert_eq!(shorthands[0].1, "foo");
        });
    }
    #[test]
    fn struct_expression_collector_should_collect_6() {
        run_after_analysis(create_program_match_6(), |tcx| {
            let hir_id = get_struct_tuple_hir_id(tcx);
            let (fields, shorthands) = collect_struct_expressions(tcx, hir_id, tup(0));

            assert_eq!(shorthands.len(), 0);
            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "123");
        });
    }
    #[test]
    fn struct_expression_collector_should_not_collect_7() {
        run_after_analysis(create_program_match_7(), |tcx| {
            let hir_id = get_struct_tuple_hir_id(tcx);
            let (fields, shorthands) = collect_struct_expressions(tcx, hir_id, tup(0));

            assert_eq!(shorthands.len(), 0);
            assert_eq!(fields.len(), 0);
        });
    }
}
