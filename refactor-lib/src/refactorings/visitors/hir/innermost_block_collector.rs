use rustc_hir::{Arm, BodyId, Block, FnDecl, HirId, ExprKind, MatchSource};
use rustc_hir::intravisit::{NestedVisitorMap, Visitor, FnKind, walk_fn, walk_expr, walk_block, walk_crate};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;

/**
 * Given a selection (byte start, byte end) and file name, this visitor finds
 * the innermost block containing `pos`
 */
struct BlockCollector<'v> {
    tcx: TyCtxt<'v>,
    pos: Span,
    body_id: Option<BodyId>,
    result: Option<BlockSelection<'v>>
}

pub fn collect_innermost_block<'v>(tcx: TyCtxt<'v>, pos: Span) -> Option<BlockSelection<'v>> {
    let mut v = BlockCollector {
        tcx,
        pos,
        body_id: None,
        result: None
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.result
}

impl<'v> Visitor<'v> for BlockCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, Self::Map> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_fn(
        &mut self,
        fk: FnKind<'v>,
        fd: &'v FnDecl,
        b: BodyId,
        s: Span,
        id: HirId,
    ) {
        self.body_id = Some(b);
        walk_fn(self, fk, fd, b, s, id);
    }

    fn visit_block(&mut self, body: &'v Block) {
        if let Some(expr) = &body.expr {
            if let ExprKind::Match(_, ref arms, MatchSource::WhileDesugar) = (*expr).kind
            {
                if let Some(arm) = arms.first() {
                    let Arm { body, .. } = arm;
                    walk_expr(self, &**body);
                }
            }
        }
        if !body.span.contains(self.pos) {
            return;
        }

        let stmts = body
            .stmts
            .iter()
            .filter(|s| self.pos.contains(s.span))
            .collect::<Vec<_>>();

        let contains_expr = body.expr.is_some() && self.pos.contains(body.expr.unwrap().span);
        let contains_stmt = stmts.len() > 0;
        
        if !contains_stmt && !contains_expr {
            walk_block(self, body);
            return;
        }

        let indexes = if contains_stmt {
            (
                body.stmts.iter().position(|s| self.pos.contains(s.span)).unwrap(),
                body.stmts.iter().rposition(|s| self.pos.contains(s.span)).unwrap()
            )
        } else {
            (0, 0)
        };

        self.result = Some( BlockSelection {
          function_body_id: self.body_id.unwrap(),
          block: body,
          contains_expr,
          contains_stmt,
          start: indexes.0,
          end: indexes.1
        });
    }
}

#[derive(Debug)]
pub struct BlockSelection<'v> {
    pub function_body_id: BodyId,
    pub block: &'v Block<'v>,
    pub contains_expr: bool,
    pub contains_stmt: bool,
    pub start: usize,
    pub end: usize
}

impl BlockSelection<'_> {
    pub fn get_span(&self) -> Span {
        let start = if self.contains_stmt {
            self.block.stmts.get(self.start).unwrap().span
        } else {
            self.block.expr.unwrap().span
        };

        let end = 
        if self.contains_expr {
            self.block.expr.unwrap().span
        } else {
            self.block.stmts.get(self.end).unwrap().span
        };

        start.with_hi(end.hi())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quote::quote;
    use quote::__rt::TokenStream;
    use crate::{create_test_span, run_after_analysis};
    use crate::refactorings::utils::get_source;

    fn create_program_1() -> TokenStream {
        quote! {
            fn foo ( ) -> i32 { bar ( ) ; baz ( ) ; { 1 } }
            fn bar() {}
            fn baz() {}
        }
    }

    const BAR_SPAN: (u32, u32) = (20, 29);
    const BAZ_SPAN: (u32, u32) = (30, 39);
    const EXPR_SPAN: (u32, u32) = (40, 45);
    
    #[test]
    fn block_collector_expr1() {
        run_after_analysis(create_program_1(), |tcx| {
            let block = collect_innermost_block(tcx, create_test_span(EXPR_SPAN.0, EXPR_SPAN.1));

            if !block.is_some() {
                panic!(get_source(tcx, create_test_span(EXPR_SPAN.0, EXPR_SPAN.1)));
            }

            let block = block.unwrap();
            assert!(block.contains_expr);
            assert!(!block.contains_stmt);
            assert_eq!(get_source(tcx, block.get_span()), "{ 1 }");
        });
    }
    #[test]
    fn block_collector_bar() {
        run_after_analysis(create_program_1(), |tcx| {
            let block = collect_innermost_block(tcx, create_test_span(BAR_SPAN.0, BAR_SPAN.1));

            if !block.is_some() {
                panic!(get_source(tcx, create_test_span(BAR_SPAN.0, BAR_SPAN.1)));
            }

            let block = block.unwrap();
            
            assert_eq!(block.start, 0);
            assert_eq!(block.end, 0);
            assert!(!block.contains_expr);
            assert!(block.contains_stmt);
            assert_eq!(get_source(tcx, block.get_span()), "bar ( ) ;");
        });
    }
    #[test]
    fn block_collector_baz() {
        run_after_analysis(create_program_1(), |tcx| {
            let block = collect_innermost_block(tcx, create_test_span(BAZ_SPAN.0, BAZ_SPAN.1));

            if !block.is_some() {
                panic!(get_source(tcx, create_test_span(BAZ_SPAN.0, BAZ_SPAN.1)));
            }

            let block = block.unwrap();

            assert_eq!(block.start, 1);
            assert_eq!(block.end, 1);
            assert!(!block.contains_expr);
            assert!(block.contains_stmt);
            assert_eq!(get_source(tcx, block.get_span()), "baz ( ) ;");
        });
    }
    #[test]
    fn block_collector_baz_expr() {
        run_after_analysis(create_program_1(), |tcx| {
            let block = collect_innermost_block(tcx, create_test_span(BAZ_SPAN.0, EXPR_SPAN.1));

            if !block.is_some() {
                panic!(get_source(tcx, create_test_span(BAZ_SPAN.0, EXPR_SPAN.1)));
            }

            let block = block.unwrap();

            assert_eq!(block.start, 1);
            assert_eq!(block.end, 1);
            assert!(block.contains_expr);
            assert!(block.contains_stmt);
            assert_eq!(get_source(tcx, block.get_span()), "baz ( ) ; { 1 }");
        });
    }
    #[test]
    fn block_collector_shouldnt_collect_cfg_omitted() {
        run_after_analysis(quote! {
            fn f ( ) { # [ cfg ( test ) ] { 1 } ; } fn f2 ( ) { # [ cfg ( not ( test ) ) ] { 1 } ; }
        }, |tcx| {
            // let cfg0 = 11;
            // let block0 = 30;
            // let block1 = 37;
            // let span1 = (block0, block1);

            let cfg10 = 52;
            // let block10 = 78;
            let block11 = 86;
            let span11 = (cfg10, block11);
            let span = span11;
            let block = collect_innermost_block(tcx, create_test_span(span.0, span.1));

            if !block.is_some() {
                panic!(get_source(tcx, create_test_span(span.0, span.1)));
            }
            let block = block.unwrap();

            assert_eq!(get_source(tcx, block.get_span()), "{ 1 } ;");
        });
    }
}