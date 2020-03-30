use rustc::ty::{self, TyCtxt};
use rustc_hir::{BodyId, ExprKind, Node};
use rustc_infer::infer::{TyCtxtInferExt};
use rustc_typeck::expr_use_visitor::{ConsumeMode, Delegate, ExprUseVisitor, Place, PlaceBase};
use rustc_span::Span;
use super::variable_use_collection::VariableUseCollection;

struct VariableCollectorDelegate<'tcx> {
    tcx: TyCtxt<'tcx>,
    extract_span: Span,
    usages: VariableUseCollection,
}

impl<'tcx> VariableCollectorDelegate<'tcx> {
    fn get_ident_and_decl_span(&self, place: &Place) -> Option<(String, Span)> {
        match place.base {
            PlaceBase::Local(lid) => {
                let decl_span = self.tcx.hir().span(lid);
                let node = self.tcx.hir().get(lid);
                if let Node::Binding(pat) = node {
                    Some((format!("{}", pat.simple_ident().unwrap()), decl_span))
                } else {
                    panic!("unhandled type"); // TODO: check which types node can be here
                }
            },
            // PlaceBase::Interior(cmt, ..) => {
            //     self.get_ident_and_decl_span(&cmt.cat)
            // },
            _ => None,
        }
    }
    fn var_used(
        &mut self,
        used_span: Span,
        place: &Place,
        is_mutated: bool,
        is_borrowed: bool
    ) {
        if let Some((ident, decl_span)) = self.get_ident_and_decl_span(place) {
            if self.extract_span.contains(used_span) && !self.extract_span.contains(decl_span) {
                // should be ret val
                self.usages.add_return_value(ident, is_borrowed, is_mutated, used_span);
            }
        }
    }
}

impl<'a, 'tcx> Delegate<'tcx> for VariableCollectorDelegate<'tcx> {
    fn consume(&mut self, place: &Place<'tcx>, _cm: ConsumeMode) {
        self.var_used(place.span, &place, false, false);
    }

    fn borrow(&mut self, place: &Place<'tcx>, bk: ty::BorrowKind) {
        let is_mutated = ty::BorrowKind::MutBorrow == bk;
        let expr = self.tcx.hir().expect_expr(place.hir_id);
        let borrow_expr = match expr.kind {
            ExprKind::AddrOf(..) => {expr},
            _ => {
                let parent = self.tcx.hir().get_parent_node(expr.hir_id);
                let parent_expr = self.tcx.hir().expect_expr(parent);
                match parent_expr.kind {
                    ExprKind::AddrOf(..) => { parent_expr},
                    _ => {panic!()}
                }
             }
        };
        self.var_used(borrow_expr.span, &place, is_mutated, true);
    }

    fn mutate(&mut self, place: &Place<'tcx>) {
        // if mode == MutateMode::Init {
        //     return;
        // }
        self.var_used(place.span, &place, true, false);
    }
}

pub fn collect_vars(tcx: rustc::ty::TyCtxt<'_>, body_id: BodyId, span: Span) -> VariableUseCollection {
    let def_id = body_id.hir_id.owner.to_def_id();
    tcx.infer_ctxt().enter(|inf| {
        let mut v = VariableCollectorDelegate {
            tcx,
            extract_span: span,
            usages: VariableUseCollection::new(),
        };
        ExprUseVisitor::new(
            &mut v,
            &inf,
            def_id,
            tcx.param_env(def_id),
            tcx.body_tables(body_id),
        )
        .consume_body(tcx.hir().body(body_id));

        v.usages
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use quote::quote;
    use crate::{create_test_span, run_after_analysis};


    #[test]
    fn closure_expr_use_visit_should_collect_zero() {
        run_after_analysis(quote! {
            fn foo ( ) { ( | | { } ) ( ) ; } 
        }, |tcx| {
            let closure_span = create_test_span(13, 28);
            let closure = collect_anonymous_closure(tcx, closure_span).unwrap();
            let vars = collect_vars(tcx, closure.body_id, closure.body_span);


            assert_eq!(0, vars.get_params().len());
        });
    }
    #[test]
    fn closure_expr_use_visit_should_collect_a() {
        run_after_analysis(quote! {
            fn foo ( ) { let i = 0 ; ( | | { & i ; } ) ( ) ; } 
        }, |tcx| {
            let closure_span = create_test_span(25, 46);
            let closure = collect_anonymous_closure(tcx, closure_span).unwrap();
            let vars = collect_vars(tcx, closure.body_id, closure.body_span);

            let borrows = vars.get_borrows();
            assert_eq!(1, borrows.len());
            assert_eq!(borrows[0], create_test_span(33, 36));

            let params = vars.get_params();
            assert_eq!(1, params.len());
            assert_eq!("i", params[0].ident);
            assert!(!params[0].is_mutated);
            assert!(!params[0].is_borrow);

            let args = vars.get_args();
            assert_eq!(1, args.len());
            assert_eq!("i", args[0].ident);
            assert!(!args[0].is_mutated);
            assert!(args[0].is_borrow);
            
        });
    }
    #[test]
    fn closure_expr_use_visit_should_collect_b() {
        run_after_analysis(quote! {
            fn foo ( ) { let i = & 0 ; ( | | { i ; } ) ( ) ; } 
        }, |tcx| {
            let closure_span = create_test_span(27, 46);
            let closure = collect_anonymous_closure(tcx, closure_span).unwrap();
            let vars = collect_vars(tcx, closure.body_id, closure.body_span);

            let borrows = vars.get_borrows();
            assert_eq!(0, borrows.len());

            let params = vars.get_params();
            assert_eq!(1, params.len());
            assert_eq!("i", params[0].ident);
            assert!(!params[0].is_mutated);
            assert!(!params[0].is_borrow);

            let args = vars.get_args();
            assert_eq!(1, args.len());
            assert_eq!("i", args[0].ident);
            assert!(!args[0].is_mutated);
            assert!(!args[0].is_borrow);
            
        });
    }
    // #[test]
    // fn closure_expr_use_visit_should_collect_c() {
    //     run_after_analysis(quote! {
    //         fn foo ( ) { let i = 0 ; ( | | { & i ; } ) ( ) ; i ; } 
    //     }, |tcx| {
    //         let closure_span = create_test_span(25, 46);
    //         let closure = collect_anonymous_closure(tcx, closure_span).unwrap();
    //         let vars = collect_vars(tcx, closure.body_id, closure.body_span);

    //         panic!("{}", get_source(tcx, tcx.hir().body(closure.body_id).value.span));

    //         let borrows = vars.get_borrows();
    //         assert_eq!(1, borrows.len());
    //         assert_eq!(borrows[0], create_test_span(33, 36));

    //         let params = vars.get_params();
    //         assert_eq!(1, params.len());
    //         assert_eq!("i", params[0].ident);
    //         assert!(!params[0].is_mutated);
    //         assert!(!params[0].is_borrow);

    //         let args = vars.get_args();
    //         assert_eq!(1, args.len());
    //         assert_eq!("i", args[0].ident);
    //         assert!(!args[0].is_mutated);
    //         assert!(args[0].is_borrow);
            
    //     });
    // }
    // #[test]
    // fn closure_expr_use_visit_should_collect_d() {
    //     run_after_analysis(quote! {
    //         fn foo ( ) { let i = 0 ; ( | | { } ) ( ) ; i ; } 
    //     }, |tcx| {
    //         let closure_span = create_test_span(25, 46);
    //         let closure = collect_anonymous_closure(tcx, closure_span).unwrap();
    //         let vars = collect_vars(tcx, closure.body_id, closure.body_span);

    //         let borrows = vars.get_borrows();
    //         assert_eq!(1, borrows.len());
    //         assert_eq!(borrows[0], create_test_span(33, 36));

    //         let params = vars.get_params();
    //         assert_eq!(1, params.len());
    //         assert_eq!("i", params[0].ident);
    //         assert!(!params[0].is_mutated);
    //         assert!(!params[0].is_borrow);

    //         let args = vars.get_args();
    //         assert_eq!(1, args.len());
    //         assert_eq!("i", args[0].ident);
    //         assert!(!args[0].is_mutated);
    //         assert!(args[0].is_borrow);
            
    //     });
    // }
}
