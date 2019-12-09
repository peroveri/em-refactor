use rustc::hir::{BodyId, Node};
use rustc::ty::{self, TyCtxt};
use rustc_typeck::expr_use_visitor::{ConsumeMode, Delegate, ExprUseVisitor, Place, PlaceBase};
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
        place: &Place,
        ty: ty::Ty<'tcx>,
        is_consumed: bool,
        is_mutated: bool,
    ) {
        if let PlaceBase::Local(lid) = place.base {
            let decl_span = self.tcx.hir().span(lid);
            let node = self.tcx.hir().get(lid);
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
    fn consume(&mut self, place: &Place<'tcx>, cm: ConsumeMode) {
        let is_consumed = if let ConsumeMode::Move = cm {
            true
        } else {
            false
        };
        self.var_used(place.span, &place, place.ty, is_consumed, false);
    }

    fn borrow(&mut self, place: &Place<'tcx>, bk: ty::BorrowKind) {
        let is_mutated = ty::BorrowKind::MutBorrow == bk;
        self.var_used(place.span, &place, place.ty, false, is_mutated);
    }

    fn mutate(&mut self, place: &Place<'tcx>) {
        // if mode == MutateMode::Init {
        //     return;
        // }
        self.var_used(place.span, &place, place.ty, false, true);
    }
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
    tcx.infer_ctxt().enter(|inf| {

        let mut v = VariableCollectorDelegate {
            tcx,
            args,
            ct: ExtractMethodContext::new(),
        };
        ExprUseVisitor::new(
            &mut v,
            &inf,
            def_id,
            tcx.param_env(def_id),
            tcx.body_tables(body_id),
        )
        .consume_body(tcx.hir().body(body_id));

        v.ct
    })
}
