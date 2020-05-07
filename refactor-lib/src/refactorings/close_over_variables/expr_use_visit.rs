use rustc_hir::{BodyId, Node};
use rustc_infer::infer::{TyCtxtInferExt};
use rustc_middle::ty::{self, TyCtxt};
use rustc_typeck::expr_use_visitor::{ConsumeMode, Delegate, ExprUseVisitor, Place, PlaceBase};
use rustc_span::Span;
use super::variable_use_collection::VariableUseCollection;
use crate::refactorings::visitors::hir::ExpressionUseKind;
use crate::refactoring_invocation::{QueryResult, RefactoringErrorInternal};

struct VariableCollectorDelegate<'tcx> {
    tcx: TyCtxt<'tcx>,
    extract_span: Span,
    usages: VariableUseCollection,
    err: QueryResult<()>
}

impl<'tcx> VariableCollectorDelegate<'tcx> {
    fn get_ident_and_decl_span(&self, place: &Place) -> QueryResult<Option<String>> {
        match place.base {
            PlaceBase::Local(local_id) => {
                let decl_span = self.tcx.hir().span(local_id);
                if self.extract_span.contains(decl_span) {
                    return Ok(None);
                }
                let node = self.tcx.hir().get(local_id);
                if let Node::Binding(pat) = node {
                    let ident = pat.simple_ident().ok_or_else(|| RefactoringErrorInternal::int(&format!("close over var / ident missing: {:?}", pat)))?;
                    Ok(Some(format!("{}", ident)))
                } else {
                    Err(RefactoringErrorInternal::int(&format!("unhandled type: {:?}", place)))
                }
            },
            // PlaceBase::Interior(cmt, ..) => {
            //     self.get_ident_and_decl_span(&cmt.cat)
            // },
            _ => Ok(None),
        }
    }
    fn var_used(
        &mut self,
        used_span: Span,
        place: &Place,
        bk: ExpressionUseKind
    ) {
        if !self.extract_span.contains(used_span) {
            return;
        }
        match self.get_ident_and_decl_span(place) {
            Ok(None) => {},
            Ok(Some(ident)) => 
                self.usages.add_return_value(ident, bk, used_span),
            Err(err) => self.err = Err(err)
        };
    }
}

impl<'a, 'tcx> Delegate<'tcx> for VariableCollectorDelegate<'tcx> {
    fn consume(&mut self, place: &Place<'tcx>, cm: ConsumeMode) {
        self.var_used(place.span, &place,  ExpressionUseKind::from_consume_mode(cm));
    }

    fn borrow(&mut self, place: &Place<'tcx>, bk: ty::BorrowKind) {
        // let expr = self.tcx.hir().expect_expr(place.hir_id);
        // let borrow_expr = match expr.kind {
        //     ExprKind::AddrOf(..) => {expr},
        //     _ => {
        //         let parent = self.tcx.hir().get_parent_node(expr.hir_id);
        //         let parent_expr = self.tcx.hir().expect_expr(parent);
        //         match parent_expr.kind {
        //             ExprKind::AddrOf(..) => { parent_expr},
        //             _ => {  panic!()} // TODO: remove panic!()
        //         }
        //      }
        // };
        self.var_used(place.span, &place, ExpressionUseKind::from_borrow_kind(bk));
    }

    fn mutate(&mut self, place: &Place<'tcx>) {
        // if mode == MutateMode::Init {
        //     return;
        // }
        self.var_used(place.span, &place, ExpressionUseKind::Mut);
    }
}

pub fn collect_vars(tcx: TyCtxt<'_>, body_id: BodyId, span: Span) -> QueryResult<VariableUseCollection> {
    let def_id = body_id.hir_id.owner.to_def_id();
    tcx.infer_ctxt().enter(|inf| {
        let mut v = VariableCollectorDelegate {
            tcx,
            extract_span: span,
            usages: VariableUseCollection::new(),
            err: Ok(())
        };
        ExprUseVisitor::new(
            &mut v,
            &inf,
            def_id,
            tcx.param_env(def_id),
            tcx.body_tables(body_id),
        )
        .consume_body(tcx.hir().body(body_id));

        v.err?;
        Ok(v.usages)
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use crate::refactoring_invocation::TyContext;
    use crate::test_utils::assert_success3;

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Vec<(ExpressionUseKind, String, (u32, u32))>> + Send> {
        Box::new(move |ty| {
            let closure = collect_anonymous_closure(ty, ty.get_span(&file_name, from, to)?).unwrap();
            let vars = collect_vars(ty.0, closure.body_id, ty.get_body_span(closure.body_id))?;

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
            (ExpressionUseKind::ImmBorrow, "i".to_string(), (56, 57))
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
        vec![(ExpressionUseKind::Copy, "i".to_string(), (56, 57))]);
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
        vec![(ExpressionUseKind::Mut, "i".to_string(), (59, 61))]);
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
            vec![(ExpressionUseKind::Mut, "i".to_string(), (58, 60))]);
    }
    #[test]
    fn closure_expr_use_visit_should_collect_e() {
        assert_success3(
r#"fn foo () {
    let s1 = "".to_string();
    let b1 = &s1;
    /*START*/(|| { 
        b1;
    })()/*END*/;
}"#, 
        map,
        vec![(ExpressionUseKind::Copy, "b1".to_string(), (87, 89))]);
    }
    #[test]
    fn closure_expr_use_visit_should_collect_f() {
        assert_success3(
r#"fn foo () {
    let mut i = 0;
    /*START*/(|| { 
        i = 1;
    })()/*END*/;
}"#, 
        map,
        vec![(ExpressionUseKind::Mut, "i".to_string(), (59, 60))]);
    }
    #[test]
    fn closure_expr_use_visit_should_collect_g() {
        assert_success3(
r#"fn foo () {
    let i = S;
    /*START*/(|| { 
        let x = i;
    })()/*END*/;
}
struct S;"#, 
        map,
        vec![(ExpressionUseKind::Move, "i".to_string(), (63, 64))]);
    }
    #[test]
    fn closure_expr_use_visit_should_collect_h() {
        assert_success3(
        r#"fn foo() {
    let j = 0;
    /*START*/(|| {
        let &(ref x) = &(&j);
    })()/*END*/;
}"#,
            map,
            vec![(ExpressionUseKind::ImmBorrow, "j".to_string(), (71, 72))]);
    }
    #[test]
    fn closure_expr_use_visit_should_collect_j() {
        assert_success3(
        r#"fn foo(s: &Box<i32>) {
    /*START*/(|| {
        b(s);
    })()/*END*/;
}
fn b(s: &i32) {}"#,
            map,
            vec![(ExpressionUseKind::ImmBorrow, "s".to_string(), (52, 53))]);
    }
    #[test]
    fn closure_expr_use_visit_should_collect_k() {
        assert_success3(
        r#"fn foo() {
    let i = S;
    /*START*/(|| {
        let _: &S = &i;
    })()/*END*/;
}
struct S;"#,
            map,
            vec![(ExpressionUseKind::ImmBorrow, "i".to_string(), (66, 67))]);
    }
    // TODO: check patterns, e.g. let _ = i;
}
