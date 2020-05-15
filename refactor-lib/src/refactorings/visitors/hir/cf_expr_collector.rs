use rustc_hir::{Arm, BodyId, Destination, Expr, ExprKind, FnDecl, HirId, MatchSource, Node };
use rustc_hir::intravisit::{NestedVisitorMap, FnKind, Visitor, walk_block, walk_expr};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::{BytePos, Span};
use super::{ControlFlowExpr, ControlFlowExprCollection};

/**
 * Given a block id / span?
 * Searches for break, continue,
 * special break-loop!
 */
struct CfExprCollector<'v> {
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

    let mut v = CfExprCollector {
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

impl CfExprCollector<'_> {
    fn points_outside(&self, dest: &Destination) -> bool {
        // TODO: span here is the whole while/for/.. expression
        let hir_id = dest.target_id.unwrap();
        let node = self.tcx.hir().get(hir_id);

        if let Node::Expr(e) = node {
            if let ExprKind::Loop(b, ..) = e.kind {
                return !b.span.shrink_to_lo().overlaps(self.block_span);
            }
        }
        false
    }
}

impl<'v> Visitor<'v> for CfExprCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.hir())
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
                let (break_span, break_ex_span) = 
                if let Some(ret_ex) = break_ex {
                    (ex.span.with_hi(ret_ex.span.lo()), Some(ret_ex.span))
                } else {
                    (ex.span, None)
                };
                if self.points_outside(&dest) {
                    self.res.push(ControlFlowExpr::brk(ex.span, break_span, break_ex_span));
                }
            },
            ExprKind::Continue(dest) => {
                if self.points_outside(&dest) {
                    self.res.push(ControlFlowExpr::cont(ex.span));
                }
            },
            ExprKind::Ret(ret_ex) => {
                let (return_span, return_expression_span) = 
                if let Some(ret_ex) = ret_ex {
                    (ex.span.with_hi(ret_ex.span.lo()), Some(ret_ex.span))
                } else {
                    (ex.span, None)
                };
                self.res.push(ControlFlowExpr::ret(ex.span, return_span, return_expression_span));
            },
            _ => {
                walk_expr(self, ex);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::run_ty_query;
    use crate::refactorings::visitors::hir::collect_innermost_contained_block;
    use crate::refactoring_invocation::{QueryResult, TyContext};
    use super::super::cf_collection::CfType;

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Vec<(String, String, CfType, Option<String>)>> + Send> {
        Box::new(move |ty| {
            let span = ty.source().map_span(&file_name, from, to)?;
            let block = collect_innermost_contained_block(ty, span).unwrap();
            let cfs = collect_cfs(ty.0, block.0.hir_id);

            Ok(cfs.items.into_iter()
                .map(|cf| (
                    ty.get_source(cf.cf_expr_span), 
                    ty.get_source(cf.cf_key_span), 
                    cf.cf_type, 
                    cf.sub_expr_span.map(|span| ty.get_source(span))))
                .collect::<Vec<_>>())
        })
    }

    #[test]
    fn should_collect_when_inside() {

        let input = r#"
        fn foo () -> i32 {
            let _ = loop { 
                let _ = /*START*/{ 
                    continue; 
                    break 1;
                    return 2; 
                    3 
                }/*END*/;
            };
            4
        }"#;
        let expected = Ok(vec![
            ("continue".to_owned(), "continue".to_owned(), CfType::Continue, None),
            ("break 1".to_owned(), "break ".to_owned(), CfType::Break, Some("1".to_owned())),
            ("return 2".to_owned(), "return ".to_owned(), CfType::Return, Some("2".to_owned())),
            ("3".to_owned(), "3".to_owned(), CfType::Nothing, None),
        ]);

        let actual = run_ty_query(input, map);

        assert_eq!(expected, actual);
    }
    #[test]
    fn should_not_collect_when_outside() {

        let input = r#"
        fn foo () -> i32 {
            /*START*/{
                let _ = loop { 
                    let _ = { 
                        continue; 
                        break 1;
                        return 2; 
                        3 
                    };
                };
                4
            }/*END*/
        }"#;
        let expected = Ok(vec![
            ("return 2".to_owned(), "return ".to_owned(), CfType::Return, Some("2".to_owned())),
            ("4".to_owned(), "4".to_owned(), CfType::Nothing, None),
        ]);

        let actual = run_ty_query(input, map);

        assert_eq!(expected, actual);
    }
    #[test]
    fn should_collect_cf_inside_expr() {

        let input = r#"
        fn foo () -> i32 {
            let _ = loop { 
                let _ = /*START*/{
                    { continue; 1 }
                }/*END*/;
            };
            4
        }"#;
        let expected = Ok(vec![
            ("continue".to_owned(), "continue".to_owned(), CfType::Continue, None),
            ("{ continue; 1 }".to_owned(), "{ continue; 1 }".to_owned(), CfType::Nothing, None),
        ]);

        let actual = run_ty_query(input, map);

        assert_eq!(expected, actual);
    }
}