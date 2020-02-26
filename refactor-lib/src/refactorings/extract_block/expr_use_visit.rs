use rustc::ty::{self, TyCtxt};
use rustc_hir::{BodyId, Node};
use rustc_infer::infer::{TyCtxtInferExt};
use rustc_typeck::expr_use_visitor::{ConsumeMode, Delegate, ExprUseVisitor, Place, PlaceBase};
use std::collections::HashMap;
use rustc_span::Span;

struct VariableCollectorDelegate<'tcx> {
    tcx: TyCtxt<'tcx>,
    extract_span: Span,
    usages: VariableUseCollection,
}

impl<'tcx> VariableCollectorDelegate<'tcx> {
    fn get_ident_and_decl_span(&self, place: &Place) -> Option<(String, Span)> {
        match place.base {
            PlaceBase::Local(lid) => {
                let decl_span = self.tcx.hir().span(lid);
                let node = self.tcx.hir().get(lid);
                if let Node::Binding(pat) = node {
                    Some((format!("{}", pat.simple_ident().unwrap()), decl_span))
                } else {
                    panic!("unhandled type"); // TODO: check which types node can be here
                }
            },
            // PlaceBase::Interior(cmt, ..) => {
            //     self.get_ident_and_decl_span(&cmt.cat)
            // },
            _ => None,
        }
    }
    fn var_used(
        &mut self,
        used_span: Span,
        place: &Place,
        is_mutated: bool,
    ) {
        if let Some((ident, decl_span)) = self.get_ident_and_decl_span(place) {
            if !self.extract_span.contains(used_span) && self.extract_span.contains(decl_span) {
                // should be ret val
                self.usages.return_values.push(VariableUse {
                    ident,
                    is_mutated,
                });
            }
        }
    }
}

impl<'a, 'tcx> Delegate<'tcx> for VariableCollectorDelegate<'tcx> {
    fn consume(&mut self, place: &Place<'tcx>, _cm: ConsumeMode) {
        self.var_used(place.span, &place, false);
    }

    fn borrow(&mut self, place: &Place<'tcx>, bk: ty::BorrowKind) {
        let is_mutated = ty::BorrowKind::MutBorrow == bk;
        self.var_used(place.span, &place, is_mutated);
    }

    fn mutate(&mut self, place: &Place<'tcx>) {
        // if mode == MutateMode::Init {
        //     return;
        // }
        self.var_used(place.span, &place, true);
    }
}

// find a name
pub struct VariableUseCollection {
    /**
     * Variables declared in 'span', used after 'span'
     */
    return_values: Vec<VariableUse>,
}
impl VariableUseCollection {
    fn new() -> Self {
        VariableUseCollection {
            return_values: vec![],
        }
    }
    pub fn get_return_values(&self) -> Vec<VariableUse> {
        let mut map: HashMap<String, VariableUse> = HashMap::new();

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
pub struct VariableUse {
    pub is_mutated: bool,
    pub ident: String,
}

pub fn collect_vars(tcx: rustc::ty::TyCtxt<'_>, body_id: BodyId, span: Span) -> VariableUseCollection {
    let def_id = body_id.hir_id.owner_def_id();
    tcx.infer_ctxt().enter(|inf| {
        let mut v = VariableCollectorDelegate {
            tcx,
            extract_span: span,
            usages: VariableUseCollection::new(),
        };
        ExprUseVisitor::new(
            &mut v,
            &inf,
            def_id,
            tcx.param_env(def_id),
            tcx.body_tables(body_id),
        )
        .consume_body(tcx.hir().body(body_id));

        v.usages
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use quote::quote;
    use crate::{create_test_span, run_after_analysis};


    #[test]
    fn expr_use_visit_should_collect_mut1() {
        run_after_analysis(quote! {
            fn foo ( ) { let i = & mut 0 ; let j = i ; mut_ ( j ) ; } 
            fn mut_(_: &mut i32) {}
        }, |tcx| {
            let body_id = collect_block(tcx, create_test_span(31, 42)).unwrap().function_body_id;
            let vars = collect_vars(tcx, body_id, create_test_span(31, 42));


            assert_eq!(1, vars.return_values.len());
            let rv = &vars.return_values[0];
            assert!(rv.is_mutated);
            assert_eq!("j", rv.ident);
        });
    }
    #[test]
    fn expr_use_visit_should_collect_borrow() {
        run_after_analysis(quote! {
            fn foo ( ) { let i = & mut 0 ; let j = i ; borrow ( j ) ; } 
            fn borrow(_: &i32) {}
        }, |tcx| {
            let body_id = collect_block(tcx, create_test_span(31, 42)).unwrap().function_body_id;
            let vars = collect_vars(tcx, body_id, create_test_span(31, 42));

            assert_eq!(1, vars.return_values.len());
            let rv = &vars.return_values[0];
            assert!(!rv.is_mutated);
            assert_eq!("j", rv.ident);
        });
    }
}
