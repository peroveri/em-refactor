use rustc_hir::{BodyId, Node};
use rustc_infer::infer::{TyCtxtInferExt};
use rustc_middle::ty::{self, TyCtxt};
use rustc_typeck::expr_use_visitor::{ConsumeMode, Delegate, ExprUseVisitor, Place, PlaceBase};
use rustc_span::Span;
use super::variable_use_collection::VariableUseCollection;
use crate::refactorings::visitors::hir::ExpressionUseKind;
use crate::refactoring_invocation::{QueryResult, RefactoringErrorInternal, TyContext};

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
                if !self.extract_span.contains(decl_span) {
                    return Ok(None);
                }

                let node = self.tcx.hir().get(local_id);
                if let Node::Binding(pat) = node {

                    Ok(Some(format!("{}", pat.simple_ident().ok_or_else(|| RefactoringErrorInternal::int(&format!("extract block / ident missing: {:?}", pat)))?)))
                } else {
                    Err(RefactoringErrorInternal::int(&format!("unhandled type: {:?}", place))) // TODO: check which types node can be here
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
        use_kind: ExpressionUseKind,
    ) {
        if self.extract_span.contains(used_span) {
            return;
        }
        match self.get_ident_and_decl_span(place) {
            Ok(None) => {},
            Ok(Some(ident)) => {
                // should be ret val
                self.usages.add_return_value(ident, use_kind);
            },
            Err(res) => self.err = Err(res)
        };

    }
}

impl<'a, 'tcx> Delegate<'tcx> for VariableCollectorDelegate<'tcx> {
    fn consume(&mut self, place: &Place<'tcx>, cm: ConsumeMode) {
        self.var_used(place.span, &place, ExpressionUseKind::from_consume_mode(cm));
    }

    fn borrow(&mut self, place: &Place<'tcx>, bk: ty::BorrowKind) {
        self.var_used(place.span, &place, ExpressionUseKind::from_borrow_kind(bk));
    }

    fn mutate(&mut self, place: &Place<'tcx>) {
        self.var_used(place.span, &place, ExpressionUseKind::Mut);
    }
}

/// Vs <- Collect variables declared inside 'span', but used outside 'span'
/// If one of Vs is a borrow or contains a borrow (struct, tuple type, etc.), then we should return an error
pub fn collect_variables_declared_in_span_and_used_later(tcx: &TyContext, body_id: BodyId, span: Span) -> QueryResult<VariableUseCollection> {
    let def_id = body_id.hir_id.owner.to_def_id();
    tcx.0.infer_ctxt().enter(|inf| {
        let mut v = VariableCollectorDelegate {
            tcx: tcx.0,
            extract_span: span,
            usages: VariableUseCollection::new(),
            err: Ok(())
        };
        ExprUseVisitor::new(
            &mut v,
            &inf,
            def_id,
            tcx.0.param_env(def_id),
            tcx.0.body_tables(body_id),
        )
        .consume_body(tcx.0.hir().body(body_id));

        v.err?;
        Ok(v.usages)
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use crate::test_utils::assert_success3;

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Vec<(String, bool)>> + Send> {
        Box::new(move |ty| {
            let span = ty.source().map_span(&file_name, from, to)?;
            let block = collect_innermost_block(ty, span).unwrap();
            let vars = collect_variables_declared_in_span_and_used_later(ty, block.1, span)?;

            Ok(vars.get_return_values().into_iter().map(|e| (e.ident, e.is_mutated)).collect::<Vec<_>>())
        })
    }

    #[test]
    fn expr_use_visit_should_collect_mut1() {
        assert_success3(
        r#"fn foo () { 
            let i = &mut 0;
            /*START*/let j = i;/*END*/
            mut_(j);
        } 
        fn mut_(_: &mut i32) {}"#,
            map,
            vec![("j".to_owned(), true)]);
    }
    #[test]
    fn expr_use_visit_should_collect_borrow() {
        assert_success3(
        r#"fn foo() {
            let i = &mut 0;
            /*START*/let j = i;/*END*/
            borrow(j);
        } 
        fn borrow(_: &i32) {}"#,
            map,
            vec![("j".to_owned(), false)]);
    }
    #[test]
    fn expr_use_visit_should_collect_borrow4() {
        assert_success3(
        r#"fn foo() {
            /*START*/let j = 0;/*END*/
            &j;
        } 
        fn borrow(_: &i32) {}"#,
            map,
            vec![("j".to_owned(), false)]);
    }
    #[test]
    fn expr_use_visit_should_collect_borrow5() {
        assert_success3(
        r#"fn foo() {
            /*START*/let j = 0;/*END*/
            let &(ref x) = &(&j);
        }"#,
            map,
            vec![("j".to_owned(), false)]);
    }
}
