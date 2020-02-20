use rustc_hir::{Arm, BodyId, Destination, Expr, ExprKind, FnDecl, HirId, MatchSource, Node };
use rustc_hir::intravisit::{NestedVisitorMap, FnKind, Visitor, walk_block, walk_expr};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::{BytePos, Span};
use super::{ControlFlowExpr, ControlFlowExprCollection};

/**
 * Given a block id / span?
 * Searches for break, continue,
 * special break-loop!
 */
struct BlockCollector<'v> {
    tcx: TyCtxt<'v>,
    block_span: Span,
    res: Vec<ControlFlowExpr>
}

pub fn collect_cfs(tcx: TyCtxt<'_>, block_hir_id: HirId) -> ControlFlowExprCollection {

    let block = if let Node::Block(b) = tcx.hir().get(block_hir_id) {
        b 
    } else {
        panic!("");
    };

    let mut v = BlockCollector {
        tcx,
        block_span: block.span,
        res: vec![]
    };

    walk_block(&mut v, block);

    if let Some(e) = block.expr {
        v.res.push(ControlFlowExpr::expr(e.span));
    } else {
        v.res.push(ControlFlowExpr::expr(block.span.with_hi(BytePos((block.span.hi().0 - 1) as u32)).shrink_to_hi()))
    }

    ControlFlowExprCollection { items: v.res }
}

impl BlockCollector<'_> {
    fn points_outside(&self, dest: &Destination) -> bool {
        // TODO: span here is the whole while/for/.. expression
        let hir_id = dest.target_id.unwrap();
        let span = self.tcx.hir().span(hir_id);
        self.block_span.overlaps(span)
    }
}

impl<'v> Visitor<'v> for BlockCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, Self::Map> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_fn(
        &mut self,
        _fk: FnKind<'v>,
        _fd: &'v FnDecl,
        _b: BodyId,
        _s: Span,
        _id: HirId,
    ) {
    }

    fn visit_expr(&mut self, ex: &'v Expr<'v>) {
        match ex.kind {
            ExprKind::Match(_, ref arms, MatchSource::WhileDesugar) => {
                if let Some(arm) = arms.first() {
                    let Arm { body, .. } = arm;
                    walk_expr(self, &**body);
                    return;
                }
            },
            // ExprKind::Match(cond, ref arms, MatchSource::IfDesugar {contains_else_clause}) => {
            //     if let Some(arm) = arms.first() {
            //         let Arm { body, .. } = arm;
            //         walk_expr(self, &**body);
            //     }
            // },
            ExprKind::Break(dest, break_ex) => {
                let break_ex_span = 
                if let Some(ret_ex) = break_ex {
                    Some(ret_ex.span)
                } else {
                    None
                };
                if self.points_outside(&dest) {
                    self.res.push(ControlFlowExpr::brk(ex.span, break_ex_span));
                }
            },
            ExprKind::Continue(dest) => {
                if self.points_outside(&dest) {
                    self.res.push(ControlFlowExpr::cont(ex.span));
                }
            },
            ExprKind::Ret(ret_ex) => {
                let ret_ex_span = 
                if let Some(ret_ex) = ret_ex {
                    Some(ret_ex.span)
                } else {
                    None
                };
                self.res.push(ControlFlowExpr::ret(ex.span, ret_ex_span));
            },
            _ => {
                walk_expr(self, ex);
            }
        }
    }
}

#[cfg(test)]
#[allow(non_upper_case_globals)]
mod test {
    use super::*;
    use super::super::*;
    use crate::{create_test_span, run_after_analysis};
    use crate::refactorings::utils::get_source;
    use quote::quote;
    use quote::__rt::TokenStream;

    fn create_program_match_1() -> TokenStream {
        quote! {
            fn foo ( ) -> i32 { let _ = loop { let _ = { continue ; break 1 ; return 2 ; 3 } ; } ; 4 }
        }
    }
    fn create_program_match_2() -> TokenStream {
        quote! {
            fn foo ( ) { while true { { continue ; } } }
        }
    }
    fn get_block<'v>(tcx: TyCtxt<'v>) -> HirId {
        collect_block(tcx, create_test_span(43, 80)).unwrap().selected_block.hir_id
    }
    fn get_block_2<'v>(tcx: TyCtxt<'v>) -> HirId {
        collect_block(tcx, create_test_span(26, 40)).unwrap().selected_block.hir_id
    }

    #[test]
    fn cf_expr_collector_should_collect_continue() {
        run_after_analysis(create_program_match_1(), |tcx| {
            let block = get_block(tcx);
            let actual = collect_cfs(tcx, block);

            for cf in &actual.items {
                if let CfType::Continue = cf.cf_type {
                    assert_eq!("continue", get_source(tcx, cf.cf_expr_span));
                    return;
                }
            }
            assert!(false);
        });
    }
    #[test]
    fn cf_expr_collector_should_collect_break() {
        run_after_analysis(create_program_match_1(), |tcx| {
            let block = get_block(tcx);
            let actual = collect_cfs(tcx, block);

            for cf in &actual.items {
                if let CfType::Break = cf.cf_type {
                    assert_eq!("break 1", get_source(tcx, cf.cf_expr_span));
                    assert_eq!("1", get_source(tcx, cf.sub_expr_span.unwrap()));
                    return;
                }
            }
            assert!(false);
        });
    }
    #[test]
    fn cf_expr_collector_should_collect_return() {
        run_after_analysis(create_program_match_1(), |tcx| {
            let block = get_block(tcx);
            let actual = collect_cfs(tcx, block);

            for cf in &actual.items {
                if let CfType::Return = cf.cf_type {
                    assert_eq!("return 2", get_source(tcx, cf.cf_expr_span));
                    assert_eq!("2", get_source(tcx, cf.sub_expr_span.unwrap()));
                    return;
                }
            }
            assert!(false);
        });
    }
    #[test]
    fn cf_expr_collector_should_collect_expr() {
        run_after_analysis(create_program_match_1(), |tcx| {
            let block = get_block(tcx);
            let actual = collect_cfs(tcx, block);

            for cf in &actual.items {
                if let CfType::Nothing = cf.cf_type {
                    assert_eq!("3", get_source(tcx, cf.cf_expr_span));
                    return;
                }
            }
            assert!(false);
        });
    }
    #[test]
    fn cf_expr_collector_should_collect_continue_2() {
        run_after_analysis(create_program_match_2(), |tcx| {
            let block = get_block_2(tcx);
            let actual = collect_cfs(tcx, block);

            assert_eq!(2, actual.items.len());
            let cf = &actual.items[0];
            assert_eq!(CfType::Continue, cf.cf_type);
            assert_eq!("continue", get_source(tcx, cf.cf_expr_span));
            let ex = &actual.items[1];
            assert_eq!(CfType::Nothing, ex.cf_type);
        });
    }
    #[test]
    fn cf_expr_collector_should_collect_continue_3() {
        run_after_analysis(quote! {
            fn foo ( ) { while true { { if false { continue ; } } } }
        }, |tcx| {
            let block = get_block_3(tcx, 26, 53);
            let actual = collect_cfs(tcx, block);

            assert_eq!(2, actual.items.len());
            let cf = &actual.items[0];
            assert_eq!(CfType::Continue, cf.cf_type);
            assert_eq!("continue", get_source(tcx, cf.cf_expr_span));
            let ex = &actual.items[1];
            assert_eq!(CfType::Nothing, ex.cf_type);
        });
    }
    fn get_block_3<'v>(tcx: TyCtxt<'v>, lo: u32, hi: u32) -> HirId {
        collect_block(tcx, create_test_span(lo, hi)).unwrap().selected_block.hir_id
    }
    #[test]
    fn cf_expr_collector_should_collect_expr_2() {
        run_after_analysis(quote! {
            fn foo ( ) { loop { let _ = { if true { } 1 } ; } }
        }, |tcx| {
            let block = get_block_3(tcx, 28, 45);
            let actual = collect_cfs(tcx, block);

            assert_eq!(1, actual.items.len());
            let ex = &actual.items[0];
            assert_eq!(CfType::Nothing, ex.cf_type);
        });
    }
}