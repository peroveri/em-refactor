use rustc::hir::{BodyId, Node};
use rustc::middle::expr_use_visitor::{ConsumeMode, Delegate, ExprUseVisitor};
use rustc::middle::mem_categorization::{cmt_, Categorization};
use rustc::ty::{self, TyCtxt};
use std::collections::HashMap;
use syntax::source_map::Span;

struct VariableCollectorDelegate<'tcx> {
    tcx: TyCtxt<'tcx>,
    extract_span: Span,
    usages: VariableUsages,
}

impl<'tcx> VariableCollectorDelegate<'tcx> {
    fn get_ident_and_decl_span(&self, cat: &Categorization) -> Option<(String, Span)> {
        match cat {
            Categorization::Local(lid) => {
                let decl_span = self.tcx.hir().span(*lid);
                let node = self.tcx.hir().get(*lid);
                if let Node::Binding(pat) = node {
                    Some((format!("{}", pat.simple_ident().unwrap()), decl_span))
                } else {
                    panic!("unhandled type"); // TODO: check which types node can be here
                }
            },
            Categorization::Interior(cmt, ..) => {
                self.get_ident_and_decl_span(&cmt.cat)
            },
            _ => None,
        }
    }
    fn var_used(
        &mut self,
        used_span: Span,
        cat: &Categorization,
        is_mutated: bool,
    ) {
        if let Some((ident, decl_span)) = self.get_ident_and_decl_span(cat) {
            if !self.extract_span.contains(used_span) && self.extract_span.contains(decl_span) {
                // should be ret val
                self.usages.return_values.push(VariableUsage {
                    ident,
                    is_mutated,
                });
            }
        }
    }
}

impl<'a, 'tcx> Delegate<'tcx> for VariableCollectorDelegate<'tcx> {
    fn consume(&mut self, cmt: &cmt_<'tcx>, _cm: ConsumeMode) {
        self.var_used(cmt.span, &cmt.cat, false);
    }

    fn borrow(&mut self, cmt: &cmt_<'tcx>, bk: ty::BorrowKind) {
        let is_mutated = ty::BorrowKind::MutBorrow == bk;
        self.var_used(cmt.span, &cmt.cat, is_mutated);
    }

    fn mutate(&mut self, cmt: &cmt_<'tcx>) {
        // if mode == MutateMode::Init {
        //     return;
        // }
        self.var_used(cmt.span, &cmt.cat, true);
    }
}

// find a name
pub struct VariableUsages {
    /**
     * Variables declared in 'span', used after 'span'
     */
    return_values: Vec<VariableUsage>,
}
impl VariableUsages {
    fn new() -> Self {
        VariableUsages {
            return_values: vec![],
        }
    }
    pub fn get_return_values(&self) -> Vec<VariableUsage> {
        let mut map: HashMap<String, VariableUsage> = HashMap::new();

        let mut ids = vec![]; // HashMap doesnt preserve order

        for rv in self.return_values.iter() {
            if !ids.contains(&rv.ident) {
                ids.push(rv.ident.to_string());
            }
            if let Some(entry) = map.get_mut(&rv.ident) {
                entry.is_mutated = entry.is_mutated || rv.is_mutated;
            } else {
                let e = rv.clone();
                map.insert(rv.ident.clone(), e);
            }
        }

        ids.iter()
            .map(|id| map.get(id).unwrap().clone())
            .collect::<Vec<_>>()
    }
}
#[derive(Clone)]
pub struct VariableUsage {
    pub is_mutated: bool,
    pub ident: String,
}

pub fn collect_vars(tcx: rustc::ty::TyCtxt<'_>, body_id: BodyId, span: Span) -> VariableUsages {
    let def_id = body_id.hir_id.owner_def_id();
    let mut v = VariableCollectorDelegate {
        tcx,
        extract_span: span,
        usages: VariableUsages::new(),
    };
    ExprUseVisitor::new(
        &mut v,
        tcx,
        def_id,
        tcx.param_env(def_id),
        tcx.region_scope_tree(def_id),
        tcx.body_tables(body_id),
    )
    .consume_body(tcx.hir().body(body_id));

    v.usages
}
