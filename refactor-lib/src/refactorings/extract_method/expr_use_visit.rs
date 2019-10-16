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
    fn var_used(
        &mut self,
        sp: Span,
        cat: &Categorization,
        ty: ty::Ty<'tcx>,
        is_consumed: bool,
        is_mutated: bool,
    ) {
        if let Categorization::Local(lid) = cat {
            let decl_span = self.tcx.hir().span(*lid);
            let node = self.tcx.hir().get(*lid);
            let ident = if let Node::Binding(pat) = node {
                format!("{}", pat.simple_ident().unwrap())
            } else {
                panic!("unhandled type"); // TODO: check which types node can be here
            };

            if self.args.spi.contains(sp) && !self.args.spi.contains(decl_span) {
                // should be arg
                self.ct.arguments.push(VariableUsage {
                    ident,
                    ty,
                    borrows: vec![],
                    was_borrow: Some(sp.lo().0),
                    is_mutated,
                    is_consumed,
                });
            } else if !self.args.spi.contains(sp) && self.args.spi.contains(decl_span) {
                // should be ret val
                self.ct.return_values.push(VariableUsage {
                    ident,
                    ty,
                    borrows: vec![],
                    was_borrow: Some(sp.lo().0),
                    is_mutated,
                    is_consumed,
                });
            }
        }
    }
}

impl<'a, 'tcx> Delegate<'tcx> for VariableCollectorDelegate<'tcx> {
    fn consume(&mut self, _: HirId, sp: Span, cmt: &cmt_<'tcx>, cm: ConsumeMode) {
        let is_consumed = if let ConsumeMode::Move(_) = cm {
            true
        } else {
            false
        };
        self.var_used(sp, &cmt.cat, cmt.ty, is_consumed, false);
    }

    fn matched_pat(&mut self, _: &Pat, _: &cmt_<'tcx>, _: MatchMode) {}

    fn consume_pat(&mut self, _: &Pat, cmt: &cmt_<'tcx>, _: ConsumeMode) {
        if let Categorization::Local(_) = cmt.cat {
            self.var_used(cmt.span, &cmt.cat, cmt.ty, true, false);
        }
    }

    fn borrow(
        &mut self,
        _: HirId,
        sp: Span,
        cmt: &cmt_<'tcx>,
        _: ty::Region<'_>,
        bk: ty::BorrowKind,
        _: LoanCause,
    ) {
        let is_mutated = ty::BorrowKind::MutBorrow == bk;
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
    return_values: Vec<VariableUsage<'tcx>>,
}
impl ExtractMethodContext<'_> {
    fn new() -> Self {
        ExtractMethodContext {
            arguments: vec![],
            return_values: vec![],
        }
    }
    pub fn get_arguments(&self) -> Vec<VariableUsage> {
        // iter?
        let mut map: HashMap<String, VariableUsage> = HashMap::new();

        for arg in self.arguments.iter() {
            if let Some(entry) = map.get_mut(&arg.ident) {
                entry.is_consumed = entry.is_consumed || arg.is_consumed;
                entry.is_mutated = entry.is_mutated || arg.is_mutated;
                if let Some(idx) = arg.was_borrow {
                    entry.borrows.push(idx);
                }
            } else {
                let mut e = arg.clone();
                if let Some(idx) = arg.was_borrow {
                    e.borrows.push(idx);
                }
                map.insert(arg.ident.clone(), e);
            }
        }

        map.into_iter()
            .map(|(_, v)| v)
            .collect::<Vec<VariableUsage>>()
    }
    pub fn get_rewrites(&self) -> Vec<u32> {
        self.get_arguments()
            .iter()
            .filter(|u| !u.is_consumed)
            .flat_map(|u| u.borrows.clone())
            .collect()
    }
    pub fn get_return_values(&self) -> Vec<VariableUsage> {
        let mut map: HashMap<String, VariableUsage> = HashMap::new();

        let mut ids = vec![]; // HashMap doesnt preserve order

        for rv in self.return_values.iter() {
            if !ids.contains(&rv.ident) {
                ids.push(rv.ident.to_string());
            }
            if let Some(entry) = map.get_mut(&rv.ident) {
                entry.is_consumed = entry.is_consumed || rv.is_consumed;
                entry.is_mutated = entry.is_mutated || rv.is_mutated;
                if let Some(idx) = rv.was_borrow {
                    entry.borrows.push(idx);
                }
            } else {
                let mut e = rv.clone();
                if let Some(idx) = rv.was_borrow {
                    e.borrows.push(idx);
                }
                map.insert(rv.ident.clone(), e);
            }
        }

        ids.iter()
            .map(|id| map.get(id).unwrap().clone())
            .collect::<Vec<_>>()

        // map.into_iter()
        //     .map(|(_, v)| v)
        //     .collect::<Vec<VariableUsage>>()
        // self.return_values
        //     .iter()
        //     .map(|(id, ty)| VariableUsage {
        //         was_borrow: None,
        //         borrows: vec![],
        //         is_consumed: false,
        //         is_mutated: false,
        //         ident: id.to_string(),
        //         ty,
        //     })
        //     .collect()
    }
}
#[derive(Clone)]
pub struct VariableUsage<'tcx> {
    pub is_consumed: bool,
    pub is_mutated: bool,
    pub was_borrow: Option<u32>, // TODO: name
    pub borrows: Vec<u32>,
    pub ident: String,
    pub ty: ty::Ty<'tcx>,
}
impl VariableUsage<'_> {
    pub fn as_param(&self) -> String {
        let mut_ = (if self.is_mutated { "mut " } else { "" }).to_string();
        let borrow = (if self.is_consumed { "" } else { "&" }).to_string();

        format!("{}: {}{}{:?}", self.ident, borrow, mut_, self.ty)
    }
    pub fn as_arg(&self) -> String {
        let mut_ = (if self.is_mutated { "mut " } else { "" }).to_string();
        let borrow = (if self.is_consumed { "" } else { "&" }).to_string();

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
