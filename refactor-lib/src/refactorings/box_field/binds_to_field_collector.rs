use rustc::hir;
use rustc::middle::expr_use_visitor::{
    ConsumeMode, Delegate, ExprUseVisitor, LoanCause, MatchMode, MutateMode,
};
use rustc::middle::mem_categorization::{cmt_, Categorization, InteriorKind};
use rustc::ty::{self, TyCtxt};
use syntax::source_map::Span;

/// (places that reads the field, writes to the field)
/// Returns (places that should be deref'ed, places that should add Box::new)
pub fn run_on_all_bodies<'tcx>(
    tcx: TyCtxt<'tcx>,
    body_ids: &[hir::BodyId],
    field_span: Span,
) -> (Vec<Span>, Vec<Span>) {
    let mut ret = (vec![], vec![]);
    for body_id in body_ids {
        let def_id = body_id.hir_id.owner_def_id();
        let mut v = BindsToFieldCollectorDelegate {
            tcx,
            field_span,
            reads: vec![],
            writes: vec![],
        };
        ExprUseVisitor::new(
            &mut v,
            tcx,
            def_id,
            tcx.param_env(def_id),
            tcx.region_scope_tree(def_id),
            tcx.body_tables(*body_id),
            None,
        )
        .consume_body(tcx.hir().body(*body_id));
        ret.0.extend(v.reads);
        ret.1.extend(v.writes);
    }
    ret
}

struct BindsToFieldCollectorDelegate<'tcx> {
    tcx: TyCtxt<'tcx>,
    field_span: Span,
    reads: Vec<Span>,
    writes: Vec<Span>,
}
// There is some query where we can get the declaration?

impl<'tcx> BindsToFieldCollectorDelegate<'tcx> {
    fn var_used(
        &mut self,
        sp: Span,
        cat: &Categorization,
        ty: ty::Ty<'tcx>,
        _is_consumed: bool,
        is_mutated: bool,
    ) {
        if let Categorization::Interior(cmt, ik) = cat {
            if let Some(adt) = cmt.ty.ty_adt_def() {
                let c = self.tcx.hir().as_local_hir_id(adt.did).unwrap();
                let struct_span = self.tcx.hir().span(c);
                if struct_span.contains(self.field_span) {
                    // eprintln!("struct span: {:?}", struct_span);
                    if is_mutated {
                        self.writes.push(sp);
                    } else {
                        self.reads.push(sp);
                    }
                }
            }
            // if ! struct.ty "equals" cmt.ty return
            // if mutated: Box::new else *
            // match ik {
            //     InteriorKind::InteriorField(_field_index) => {
            //         eprintln!("cmt: {}, ty: {}, fi", cmt.ty, ty);
            //     }
            //     InteriorKind::InteriorElement(offset_kind) => {
            //         eprintln!("{}, ok: {:?}", cmt.ty, offset_kind);
            //     }
            // };

            // let decl_span = self.tcx.hir().span(*lid);
            // let node = self.tcx.hir().get(*lid);
            // let ident = if let Node::Binding(pat) = node {
            //     format!("{}", pat.simple_ident().unwrap())
            // } else {
            //     panic!("unhandled type"); // TODO: check which types node can be here
            // };

            // if self.args.spi.contains(sp) && !self.args.spi.contains(decl_span) {
            //     // should be arg
            //     self.ct.arguments.push(VariableUsage {
            //         ident,
            //         ty,
            //         borrows: vec![],
            //         was_borrow: Some(sp.lo().0),
            //         is_mutated,
            //         is_consumed,
            //     });
            // } else if !self.args.spi.contains(sp) && self.args.spi.contains(decl_span) {
            //     // should be ret val
            //     self.ct.return_values.push(VariableUsage {
            //         ident,
            //         ty,
            //         borrows: vec![],
            //         was_borrow: Some(sp.lo().0),
            //         is_mutated,
            //         is_consumed,
            //     });
            // }
        }
    }
}

impl<'a, 'tcx> Delegate<'tcx> for BindsToFieldCollectorDelegate<'tcx> {
    fn consume(&mut self, _: hir::HirId, sp: Span, cmt: &cmt_<'tcx>, cm: ConsumeMode) {
        let is_consumed = if let ConsumeMode::Move(_) = cm {
            true
        } else {
            false
        };
        // self.var_used(sp, &cmt.cat, cmt.ty, is_consumed, false);
    }

    fn matched_pat(&mut self, _: &hir::Pat, _: &cmt_<'tcx>, _: MatchMode) {}

    fn consume_pat(&mut self, _: &hir::Pat, cmt: &cmt_<'tcx>, _: ConsumeMode) {
        if let Categorization::Local(_) = cmt.cat {
            // self.var_used(cmt.span, &cmt.cat, cmt.ty, true, false);
        }
    }

    fn borrow(
        &mut self,
        _: hir::HirId,
        sp: Span,
        cmt: &cmt_<'tcx>,
        _: ty::Region<'_>,
        bk: ty::BorrowKind,
        _: LoanCause,
    ) {
        // eprintln!("borrow: {}", super::super::utils::get_source(self.tcx, sp));
        let is_mutated = ty::BorrowKind::MutBorrow == bk;
        // self.var_used(sp, &cmt.cat, cmt.ty, false, is_mutated);
    }

    fn mutate(&mut self, hirid: hir::HirId, sp: Span, cmt: &cmt_<'tcx>, mode: MutateMode) {
        // eprintln!("mutate - sp: {}, cmt: {:?}", super::super::utils::get_source(self.tcx, sp), cmt);
        let hirsrc = super::super::utils::get_source(self.tcx, self.tcx.hir().span(hirid));
        eprintln!(
            "mutate: {},\tcmt:{},\tmode: {:?}\thir: {}",
            super::super::utils::get_source(self.tcx, sp),
            super::super::utils::get_source(self.tcx, cmt.span),
            mode,
            hirsrc
        );
        if mode == MutateMode::Init {
            return;
        }
        self.var_used(sp, &cmt.cat, cmt.ty, false, true);
    }

    fn decl_without_init(&mut self, _: hir::HirId, _: Span) {}
}
