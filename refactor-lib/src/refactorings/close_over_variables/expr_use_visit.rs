use rustc::ty::{self, TyCtxt};
use rustc_hir::{BodyId, ExprKind, Node};
use rustc_infer::infer::{TyCtxtInferExt};
use rustc_typeck::expr_use_visitor::{ConsumeMode, Delegate, ExprUseVisitor, Place, PlaceBase};
use rustc_span::Span;
use super::variable_use_collection::VariableUseCollection;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Bk {
    Copy,
    Move,
    ImmBorrow,
    UniqueImmBorrow,
    MutBorrow,
    Mut
}
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
        bk: Bk
    ) {
        if let Some((ident, decl_span)) = self.get_ident_and_decl_span(place) {
            if self.extract_span.contains(used_span) && !self.extract_span.contains(decl_span) {
                // should be ret val
                self.usages.add_return_value(ident, bk, used_span);
            }
        }
    }
}

fn cm_to_bk(cm: ConsumeMode) -> Bk{
    match cm {
        ConsumeMode::Copy => Bk::Copy,
        ConsumeMode::Move => Bk::Move
    }
}
fn bk_to_bk(cm: ty::BorrowKind) -> Bk{ // Change name
    match cm {
        ty::BorrowKind::ImmBorrow => Bk::ImmBorrow,
        ty::BorrowKind::UniqueImmBorrow => Bk::UniqueImmBorrow,
        ty::BorrowKind::MutBorrow => Bk::MutBorrow,
    }
}

impl<'a, 'tcx> Delegate<'tcx> for VariableCollectorDelegate<'tcx> {
    fn consume(&mut self, place: &Place<'tcx>, cm: ConsumeMode) {
        self.var_used(place.span, &place, cm_to_bk(cm));
    }

    fn borrow(&mut self, place: &Place<'tcx>, bk: ty::BorrowKind) {
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
        self.var_used(borrow_expr.span, &place, bk_to_bk(bk));
    }

    fn mutate(&mut self, place: &Place<'tcx>) {
        // if mode == MutateMode::Init {
        //     return;
        // }
        self.var_used(place.span, &place, Bk::Mut);
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
    use crate::refactoring_invocation::TyContext;
    use crate::test_utils::assert_success3;

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Vec<(Bk, String, (u32, u32))>> + Send> {
        Box::new(move |ty| {
            let closure = collect_anonymous_closure(ty.0, ty.get_span(&file_name, from, to)?).unwrap();
            let vars = collect_vars(ty.0, closure.body_id, closure.body_span);

            Ok(vars.to_cmp())
        })
    }

    #[test]
    fn closure_expr_use_visit_should_collect_zero() {
        assert_success3(
            r#"fn foo () {
    /*START*/(|| { })()/*END*/;
}"#, 
        map, 
        vec![]);
    }
    #[test]
    fn closure_expr_use_visit_should_collect_a() {
        assert_success3(
            r#"fn foo () {
    let i = 0;
    /*START*/(|| { 
        &i;
    })()/*END*/;
}"#,
        map,
        vec![
            (Bk::ImmBorrow, "i".to_string(), (55, 57))
        ]);
    }
    #[test]
    fn closure_expr_use_visit_should_collect_b() {
        assert_success3(
            r#"fn foo () {
    let i = &0;
    /*START*/(|| { 
        i; 
    })()/*END*/;
}"#, 
        map,
        vec![(Bk::Copy, "i".to_string(), (56, 57))]);
    }
    #[test]
    fn closure_expr_use_visit_should_collect_c() {
        assert_success3(
            r#"fn foo () {
    let i = &mut 0;
    /*START*/(|| {
        *i = 1;
    })()/*END*/;
}"#,
        map,
        vec![(Bk::Mut, "i".to_string(), (59, 61))]);
    }
    #[test]
    fn closure_expr_use_visit_should_collect_d() {
        assert_success3(
            r#"fn foo() {
    let i = &mut 0;
    /*START*/(|| {
        *i = 1;
    })()/*END*/;
}"#, map,
            vec![(Bk::Mut, "i".to_string(), (58, 60))]);
    }
}
