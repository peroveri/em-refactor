use rustc_hir::{Node, PatKind};
use rustc_middle::ty::{self, TyCtxt};
use rustc_typeck::expr_use_visitor::{ConsumeMode, Delegate, Place, PlaceBase};
use rustc_span::Span;
// use super::get_source;

struct VariableCollectorDelegate<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> VariableCollectorDelegate<'tcx> {
    fn get_ident_and_decl_span(&self, place: &Place) {
        match &place.base {
            PlaceBase::Local(lid) => {
                // let decl_span = self.tcx.hir().span(*lid);
                let node = self.tcx.hir().get(*lid);
                // eprintln!("node: {:?}, span: {}, {:?}", node, get_source(self.tcx, decl_span), decl_span);
                if let Node::Binding(pat) = node {
                    if let PatKind::Binding(_, _hir_id, ..) = pat.kind {
                        // let node = self.tcx.hir().get(hir_id);
                        // eprintln!("-- node: {:?}", node);
                        // let expr = self.tcx.hir().expect_expr(hir_id);
                        // if let ExprKind::Path(qpath) = &expr.kind {
                        // }
                    }
                    // eprintln!("-- binding: {:?}, {:?}, kind: {:?}", pat, pat.is_refutable(), pat.kind);
                    
                } else {
                    panic!("unhandled type"); // TODO: check which types node can be here
                }
                // eprintln!();
            },
            _e => {
                // eprintln!("place: {:?}, base: {:?}", &place, &place.base);
            },
        }
    }
    fn var_used(
        &mut self,
        _used_span: Span,
        place: &Place,
        _is_mutated: bool,
    ) {
        self.get_ident_and_decl_span(place);
    }
}

impl<'a, 'tcx> Delegate<'tcx> for VariableCollectorDelegate<'tcx> {
    fn consume(&mut self, place: &Place<'tcx>, _cm: ConsumeMode) {
        self.var_used(place.span, &place, false);
    }

    fn borrow(&mut self, place: &Place<'tcx>, bk: ty::BorrowKind) {
        let is_mutated = ty::BorrowKind::MutBorrow == bk;
        self.var_used(place.span, &place, is_mutated);
    }

    fn mutate(&mut self, place: &Place<'tcx>) {
        // if mode == MutateMode::Init {
        //     return;
        // }
        self.var_used(place.span, &place, true);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quote::quote;
    use crate::{create_test_span, run_after_analysis};
    use super::super::super::visitors::hir::collect_innermost_block;
    use crate::refactoring_invocation::TyContext;
    use rustc_hir::BodyId;
    use rustc_infer::infer::{TyCtxtInferExt};
    use rustc_typeck::expr_use_visitor::ExprUseVisitor;

    pub fn collect_vars(tcx: TyCtxt<'_>, body_id: BodyId) {
        let def_id = body_id.hir_id.owner.to_def_id();
        tcx.infer_ctxt().enter(|inf| {
            let mut v = VariableCollectorDelegate {
                tcx,
            };
            ExprUseVisitor::new(
                &mut v,
                &inf,
                def_id,
                tcx.param_env(def_id),
                tcx.body_tables(body_id),
            )
            .consume_body(tcx.hir().body(body_id));
        })
    }
    
    #[test]
    fn expr_use_visit_should_collect_mut1() {
        run_after_analysis( quote! {
            fn foo ( s1 : S ) { if let S { f , g : 1 } | S { f : 1, g : f } = s1 { let _ : i32 = f ; } } struct S { f : i32 , g : i32 }
        }, |tcx| {
            let (_, body_id) = collect_innermost_block(&TyContext(tcx), create_test_span(20, 91)).unwrap();
            collect_vars(tcx, body_id);
        });
    }
}
