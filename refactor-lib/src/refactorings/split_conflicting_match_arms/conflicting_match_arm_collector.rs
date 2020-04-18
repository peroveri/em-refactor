use rustc_hir::{BodyId, Expr, ExprKind, FnDecl, HirId};
use rustc_hir::intravisit::{FnKind, walk_expr, walk_fn, walk_crate, NestedVisitorMap, Visitor};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use if_chain::if_chain;

///
/// Collect all places where a given struct field occurs in a match pattern 
/// and where it introduces a conditional binding to the field and another local
/// 
/// Walk patterns
/// If or-pattern: collect conflicting bindings in those, resolving to the field
/// for each conflicting
///     if parent is match:
///        append (parent, patterns, body, if-guard) to result
///     else:
///        append (parent, patterns) to errors
/// 
/// Walk match expr
/// if sugar: find conflicting pat with reject
/// else: find conflicting with replace
/// Walk fn: find conflicting with reject
/// 
/// 
/// # Example TODO
/// ``` 
/// match EXPR {
/// S {f, g: _} | S{f: _, g: f}
/// }
/// ```
pub fn collect_conflicting_match_arms(
    tcx: TyCtxt,
    struct_hir_id: HirId,
    field_index: usize,
) -> Vec<Span> {
    let mut v = ConflictingMatchArmCollector {
        tcx,
        struct_hir_id,
        field: vec![],
        field_index,
        body_id: None,
    };
    
    walk_crate(&mut v, tcx.hir().krate());

    v.field
}

// pub struct ArmResults {
//     arms: Vec<ArmResult>
// }
// pub struct ArmResult {
//     parent: MatchExpr,
//     if_guard: Option<Expr>,
//     body: Expr,
//     patterns: Vec<Pattern>
// }

struct ConflictingMatchArmCollector<'v> {
    tcx: TyCtxt<'v>,
    struct_hir_id: HirId,
    field: Vec<Span>,
    field_index: usize,
    body_id: Option<BodyId>,
}

impl ConflictingMatchArmCollector<'_> {
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

impl<'v> Visitor<'v> for ConflictingMatchArmCollector<'v> {
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
    use crate::{create_test_span, run_after_analysis};
    use crate::refactorings::visitors::collect_field;
    use super::super::super::utils::get_source;
    use quote::quote;
    use quote::__private::TokenStream;

    fn create_program_match_1() -> TokenStream {
        quote! {
            fn foo ( s1 : S ) { if let S { f , g : 1 } | S { f : 1 , g : f } = s1 { let _ : i32 = f } } struct S { f : i32 , g : i32 }
        }
    }

    fn get_struct_tuple_hir_id(tcx: TyCtxt<'_>) -> HirId {
        let (field, _) = collect_field(tcx, create_test_span(103, 104)).unwrap();
        let struct_def_id = field.hir_id.owner.to_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }

    #[test]
    #[ignore]
    fn struct_expression_collector_should_collect_1() {
        run_after_analysis(create_program_match_1(), |tcx| {
            let hir_id = get_struct_tuple_hir_id(tcx);
            let fields = collect_conflicting_match_arms(tcx, hir_id, 0);

            assert_eq!(fields.len(), 1);
            assert_eq!(get_source(tcx, fields[0]), "123");
        });
    }
    // #[test]
    // fn struct_expression_collector_x() {
    //     run_after_analysis(quote!{
    //         struct S { f : i32 , g : i32 }
    //         fn f(s1: S) {
    //             match s1 {
    //                 S {f: f, g: _} | S {f: _, g: f} => {}
    //             }
    //         }
    //     }, |tcx| {
    //         let hir_id = get_struct_tuple_hir_id(tcx);
    //         let fields = collect_conflicting_match_arms(tcx, hir_id, 0);

    //         assert_eq!(fields.len(), 1);
    //         assert_eq!(get_source(tcx, fields[0]), "123");
    //     });
    // }
    // #[test]
    // fn struct_expression_collector_y() {
    //     run_after_analysis(quote!{
    //         struct S { f : i32 , g : i32 }
    //         fn f(s1: S) {
    //             match s1 {
    //                 S {f: f, g: _} | S {f: _, g: g} => {}
    //             }
    //         }
    //     }, |tcx| {
    //     });
    // }
}
