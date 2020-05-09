use rustc_hir::{BodyId, Node, hir_id::HirId};
use rustc_infer::infer::{TyCtxtInferExt};
use rustc_middle::ty::{self, TyCtxt};
use rustc_typeck::expr_use_visitor::{ConsumeMode, Delegate, ExprUseVisitor, Place, PlaceBase};
use rustc_span::Span;
use crate::refactorings::visitors::hir::ExpressionUseKind;
use crate::refactoring_invocation::{QueryResult, RefactoringErrorInternal};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TypeKind {
    Mut,
    Borrow,
    None
}

struct VariableCollectorDelegate<'tcx> {
    tcx: TyCtxt<'tcx>,
    extract_span: Span,
    usages: Vec<(HirId, String, ExpressionUseKind, String, TypeKind)>,
    err: QueryResult<()>
}

impl<'tcx> VariableCollectorDelegate<'tcx> {
    fn get_ident_and_decl_span(&mut self, place: &Place, bk: ExpressionUseKind) -> QueryResult<()> {
        match place.base {
            PlaceBase::Local(local_id) => {
                let decl_span = self.tcx.hir().span(local_id);
                if self.extract_span.contains(decl_span) {
                    return Ok(());
                }
                let node = self.tcx.hir().get(local_id);
                if let Node::Binding(pat) = node {
                    let ident = pat.simple_ident().ok_or_else(|| RefactoringErrorInternal::int(&format!("close over var / ident missing: {:?}", pat)))?;

                    let (type_, type_kind) = self.get_type(pat);
                    // let old_type = self.format_ty(&place.ty);
                    self.usages.push((local_id, format!("{}", ident), bk, type_, type_kind));
                } else {
                    return Err(RefactoringErrorInternal::int(&format!("unhandled type: {:?}", place)));
                }
            },
            _ => {},
        }
        Ok(())
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
        match self.get_ident_and_decl_span(place, bk) {
            Err(err) => self.err = Err(err),
            _ => {}
        };
    }
    fn get_type(&self, pat: &rustc_hir::Pat) -> (String, TypeKind) {
        
        let typecheck_table = self.tcx.typeck_tables_of(pat.hir_id.owner.to_def_id());
        if let Some(pat_type) = typecheck_table.pat_ty_opt(pat) {

            let kind = match pat_type.kind {
                rustc_middle::ty::TyKind::Ref(.., rustc_middle::mir::Mutability::Mut) => TypeKind::Mut,
                rustc_middle::ty::TyKind::Ref(.., rustc_middle::mir::Mutability::Not) => TypeKind::Borrow,
                _ => TypeKind::None
            };

            return (format!("{}", pat_type), kind);
        }


        ("".to_owned(), TypeKind::None) // return err?
    }
}

impl<'a, 'tcx> Delegate<'tcx> for VariableCollectorDelegate<'tcx> {
    fn consume(&mut self, place: &Place<'tcx>, cm: ConsumeMode) {
        self.var_used(place.span, &place,  ExpressionUseKind::from_consume_mode(cm));
    }

    fn borrow(&mut self, place: &Place<'tcx>, bk: ty::BorrowKind) {
        self.var_used(place.span, &place, ExpressionUseKind::from_borrow_kind(bk));
    }

    fn mutate(&mut self, place: &Place<'tcx>) {
        self.var_used(place.span, &place, ExpressionUseKind::Mut);
    }
}

pub fn collect_vars(tcx: TyCtxt<'_>, body_id: BodyId) -> QueryResult<Vec<(HirId, String, ExpressionUseKind, String, TypeKind)>> {
    let def_id = body_id.hir_id.owner.to_def_id();
    let body = tcx.hir().body(body_id);
    tcx.infer_ctxt().enter(|inf| {
        let mut v = VariableCollectorDelegate {
            tcx,
            extract_span: body.value.span,
            usages: vec![],
            err: Ok(())
        };
        ExprUseVisitor::new(
            &mut v,
            &inf,
            def_id,
            tcx.param_env(def_id),
            tcx.body_tables(body_id),
        ).consume_body(body);

        v.err?;
        Ok(v.usages)
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use crate::refactoring_invocation::TyContext;
    use crate::test_utils::run_ty_query;

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Vec<(String, ExpressionUseKind, String, TypeKind)>> + Send> {
        Box::new(move |ty| {
            let span = ty.source().map_span(&file_name, from, to)?;
            let closure = collect_anonymous_closure(ty, span)?;
            Ok(collect_vars(ty.0, closure.body_id)?.into_iter()
                .map(|(_, a, b,c ,d)| (a,b,c,d))
                .collect::<Vec<_>>())
        })
    }

    #[test]
    #[ignore]
    fn should_collect_submod() {

        let input = r#"
mod submod {
    fn foo(s: S) {
        /*START*/(|| {
            &s;
        })()/*END*/;
    }
    pub struct S;
}"#;
        let expected = Ok(vec![
            ("s".to_owned(), ExpressionUseKind::ImmBorrow, "S".to_owned(), TypeKind::None)
        ]);

        let actual = run_ty_query(input, map);

        assert_eq!(expected, actual);
    }
}
