use rustc::hir::{BodyId, HirId, Node, Pat};
use rustc::middle::expr_use_visitor::{
    ConsumeMode, Delegate, ExprUseVisitor, LoanCause, MatchMode, MutateMode,
};
use rustc::middle::mem_categorization::{cmt_, Categorization};
use rustc::ty::{self, TyCtxt};
use std::collections::HashMap;
use syntax::source_map::Span;

struct VariableCollectorDelegate<'tcx> {
    tcx: TyCtxt<'tcx>,
    args: CollectVarsArgs,
    ct: ExtractMethodContext<'tcx>,
}

impl<'tcx> VariableCollectorDelegate<'tcx> {
    fn var_used(&mut self, sp: Span, cat: &Categorization, ty: ty::Ty<'tcx>, is_consumed: bool, is_mutated: bool) {
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
                self.ct.arguments.push(VariableUsage {
                    ident, 
                    ty,
                    is_mutated,
                    is_consumed
                });
            } else if !self.args.spi.contains(sp) && self.args.spi.contains(decl_span) {
                // should be ret val
                self.ct.return_values.push((ident, ty));
            }
        }
    }
}

impl<'a, 'tcx> Delegate<'tcx> for VariableCollectorDelegate<'tcx> {
    fn consume(&mut self, _: HirId, sp: Span, cmt: &cmt_<'tcx>, cm: ConsumeMode) {
        let is_consumed = if let ConsumeMode::Move(_) = cm { true} else {false};
        self.var_used(sp, &cmt.cat, cmt.ty, is_consumed, false);
    }

    fn matched_pat(&mut self, _: &Pat, _: &cmt_<'tcx>, _: MatchMode) {}

    fn consume_pat(&mut self, _: &Pat, _: &cmt_<'tcx>, _: ConsumeMode) {}

    fn borrow(
        &mut self,
        _: HirId,
        sp: Span,
        cmt: &cmt_<'tcx>,
        _: ty::Region<'_>,
        bk: ty::BorrowKind,
        _: LoanCause,
    ) {
        let is_mutated = if let ty::BorrowKind::MutBorrow = bk {true} else {false};
        self.var_used(sp, &cmt.cat, cmt.ty, false, is_mutated);
    }

    fn mutate(&mut self, _: HirId, sp: Span, cmt: &cmt_<'tcx>, mode: MutateMode) {
        if mode == MutateMode::Init {
            return;
        }
        self.var_used(sp, &cmt.cat, cmt.ty, false, true);
    }

    fn decl_without_init(&mut self, _: HirId, _: Span) {}
}

// find a name
pub struct ExtractMethodContext<'tcx> {
    /**
     * Variables declared in S0, used in Si
     */
    arguments: Vec<VariableUsage<'tcx>>,
    /**
     * Variables declared in Si, used in Sj
     */
    return_values: Vec<(String, ty::Ty<'tcx>)>,
}
impl ExtractMethodContext<'_> {
    fn new() -> Self {
        ExtractMethodContext {
            arguments: vec![],
            return_values: vec![],
        }
    }
    pub fn get_arguments(&self) -> Vec<VariableUsage> { // iter?
        let mut map: HashMap<String, VariableUsage> = HashMap::new();

        for arg in self.arguments.iter() {
            if let Some(entry) = map.get_mut(&arg.ident) {
                entry.is_consumed = entry.is_consumed || arg.is_consumed;
                entry.is_mutated = entry.is_mutated || arg.is_mutated;
            } else {
                map.insert(arg.ident.clone(), arg.clone());
            }
        }

        map.into_iter().map(|(k,v)| v).collect::<Vec<VariableUsage>>()
    }
    pub fn get_return_values(&self) -> Vec<VariableUsage> {
        self.return_values.iter().map(|(id, ty)| VariableUsage {
            is_consumed: false,
            is_mutated: false,
            ident: id.to_string(),
            ty
        }).collect()
    }
}
#[derive(Clone)]
pub struct VariableUsage<'tcx> {
    pub is_consumed: bool,
    pub is_mutated: bool,
    pub ident: String,
    pub ty: ty::Ty<'tcx>
}
impl VariableUsage<'_> {
    pub fn as_param(&self) -> String {
        let mut_ = (if self.is_mutated {"mut "} else {""}).to_string();
        let borrow = (if self.is_consumed {""} else {"&"}).to_string();

        format!("{}: {}{}{:?}", self.ident, borrow, mut_, self.ty)
    }
    pub fn as_arg(&self) -> String {
        let mut_ = (if self.is_mutated {"mut "} else {""}).to_string();
        let borrow = (if self.is_consumed {""} else {"&"}).to_string();

        format!("{}{}{}", borrow, mut_, self.ident)
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
