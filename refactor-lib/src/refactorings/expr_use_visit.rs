use rustc::hir::{BodyId, HirId, Node, Pat};
use rustc::middle::expr_use_visitor::{
    ConsumeMode, Delegate, ExprUseVisitor, LoanCause, MatchMode, MutateMode,
};
use rustc::middle::mem_categorization::{cmt_, Categorization};
use rustc::ty::{self, TyCtxt};
use syntax::source_map::Span;

struct VariableCollectorDelegate<'tcx> {
    tcx: TyCtxt<'tcx>,
    args: CollectVarsArgs,
    ct: ExtractMethodContext<'tcx>,
}

impl<'tcx> VariableCollectorDelegate<'tcx> {
    fn var_used(&mut self, sp: Span, cat: &Categorization, ty: ty::Ty<'tcx>) {
        if let Categorization::Local(lid) = cat {
            let decl_span = self.tcx.hir().span(*lid);
            let node = self.tcx.hir().get(*lid);
            let ident = if let Node::Binding(pat) = node {
                format!("{}", pat.simple_ident().unwrap())
            } else {
                panic!("unhandled type"); // TODO: check which types node can be here
            };

            // println!("var_used decl: {:?}, used: {:?}, ident: {}", decl_span, sp, ident);
            if self.args.spi.contains(sp) && !self.args.spi.contains(decl_span) {
                // should be arg
                self.ct.arguments.push((ident, ty));
            } else if !self.args.spi.contains(sp) && self.args.spi.contains(decl_span) {
                // should be ret val
                self.ct.return_values.push((ident, ty));
            }
        }
    }
}

impl<'a, 'tcx> Delegate<'tcx> for VariableCollectorDelegate<'tcx> {
    fn consume(&mut self, _: HirId, sp: Span, cmt: &cmt_<'tcx>, _: ConsumeMode) {
        self.var_used(sp, &cmt.cat, cmt.ty);
    }

    fn matched_pat(&mut self, _: &Pat, _: &cmt_<'tcx>, _: MatchMode) {}

    fn consume_pat(&mut self, _: &Pat, _: &cmt_<'tcx>, _: ConsumeMode) {}

    fn borrow(
        &mut self,
        _: HirId,
        sp: Span,
        cmt: &cmt_<'tcx>,
        _: ty::Region<'_>,
        _: ty::BorrowKind,
        _: LoanCause,
    ) {
        self.var_used(sp, &cmt.cat, cmt.ty);
    }

    fn mutate(&mut self, _: HirId, sp: Span, cmt: &cmt_<'tcx>, mode: MutateMode) {
        if mode == MutateMode::Init {
            return;
        }
        self.var_used(sp, &cmt.cat, cmt.ty);
    }

    fn decl_without_init(&mut self, _: HirId, _: Span) {}
}

// find a name
pub struct ExtractMethodContext<'tcx> {
    /**
     * Variables declared in S0, used in Si
     */
    pub arguments: Vec<(String, ty::Ty<'tcx>)>,
    /**
     * Variables declared in Si, used in Sj
     */
    pub return_values: Vec<(String, ty::Ty<'tcx>)>,
}
impl ExtractMethodContext<'_> {
    fn new() -> Self {
        ExtractMethodContext {
            arguments: vec![],
            return_values: vec![],
        }
    }
}
pub struct CollectVarsArgs {
    pub body_id: BodyId,
    // pub sp0: Span,
    pub spi: Span,
    // pub spj: Span,
}

pub fn collect_vars(tcx: rustc::ty::TyCtxt<'_>, args: CollectVarsArgs) -> ExtractMethodContext<'_> {
    let body_id = args.body_id;
    let def_id = body_id.hir_id.owner_def_id();
    let mut v = VariableCollectorDelegate {
        tcx,
        args,
        ct: ExtractMethodContext::new(),
    };
    ExprUseVisitor::new(
        &mut v,
        tcx,
        def_id,
        tcx.param_env(def_id),
        tcx.region_scope_tree(def_id),
        tcx.body_tables(body_id),
        None,
    )
    .consume_body(tcx.hir().body(body_id));

    v.ct
}
