use rustc::hir;
use rustc::middle::expr_use_visitor::{ConsumeMode, Delegate, ExprUseVisitor};
use rustc::middle::mem_categorization::{cmt_, Categorization};
use rustc::ty::{self, TyCtxt};
use syntax::source_map::Span;

/// Returns (places that should be deref'ed, places that should add Box::new)
pub fn run_on_all_bodies<'tcx>(
    tcx: TyCtxt<'tcx>,
    body_ids: &[hir::BodyId],
    field_span: Span,
    field_name: String
) -> (Vec<Span>, Vec<Span>) {
    let mut ret = (vec![], vec![]);
    for body_id in body_ids {
        let def_id = body_id.hir_id.owner_def_id();
        let mut v = BindsToFieldCollectorDelegate {
            tcx,
            field_span,
            field_name: field_name.to_string(),
            references: vec![],
            inits: vec![],
        };
        ExprUseVisitor::new(
            &mut v,
            tcx,
            def_id,
            tcx.param_env(def_id),
            tcx.region_scope_tree(def_id),
            tcx.body_tables(*body_id),
        )
        .consume_body(tcx.hir().body(*body_id));
        ret.0.extend(v.references);
        ret.1.extend(v.inits);
    }
    ret
}

struct BindsToFieldCollectorDelegate<'tcx> {
    tcx: TyCtxt<'tcx>,
    field_span: Span,
    field_name: String,
    references: Vec<Span>,
    inits: Vec<Span>,
}

impl<'tcx> BindsToFieldCollectorDelegate<'tcx> {
    
    fn is_struct(&self, ty: ty::Ty) -> bool {
        if let Some(adt) = ty.ty_adt_def() {
            if let Some(c) = self.tcx.hir().as_local_hir_id(adt.did) {
                let struct_span = self.tcx.hir().span(c);
                return struct_span.contains(self.field_span);
            }
        }
        false
    }
    fn is_field(&self, cmt: &cmt_) -> bool {
        // TODO: use if_chain macro?
        if let Categorization::Interior(struct_cmt, _) = &cmt.cat {
            if let Some(adt) = struct_cmt.ty.ty_adt_def() {
                if let Some(c) = self.tcx.hir().as_local_hir_id(adt.did) {
                    let struct_span = self.tcx.hir().span(c);
                    if struct_span.contains(self.field_span) {
                        return true;
                    }
                }
            }
        }
        false
    }
    fn add_ref_if_is_field(&mut self, cmt: &cmt_) {
        if self.is_field(cmt) {
            self.references.push(cmt.span);
        }
    }
    fn add_init_if_is_struct(&mut self, cmt: &cmt_) {
        // TODO: check where it is used, shouldn't add box::new in pattern matching
        if self.is_struct(&cmt.ty) {
            let expr = self.tcx.hir().expect_expr(cmt.hir_id);
            if let hir::ExprKind::Struct(_, fields, _) =  &expr.kind {
                for field in fields {
                    if format!("{}", field.ident) == self.field_name {
                        self.inits.push(field.expr.span);
                    }
                }
            }
        }
    }
}

impl<'a, 'tcx> Delegate<'tcx> for BindsToFieldCollectorDelegate<'tcx> {
    fn consume(&mut self, cmt: &cmt_<'tcx>, _cm: ConsumeMode) {
        self.add_ref_if_is_field(cmt);
        self.add_init_if_is_struct(cmt);
    }

    fn borrow(&mut self, cmt: &cmt_<'tcx>, _bk: ty::BorrowKind) {
        self.add_ref_if_is_field(cmt);
    }

    fn mutate(&mut self, cmt: &cmt_<'tcx>) {
        self.add_ref_if_is_field(cmt);
    }
}
