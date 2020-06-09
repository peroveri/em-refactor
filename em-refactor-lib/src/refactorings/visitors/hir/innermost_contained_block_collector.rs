use rustc_hir::{Block, BodyId, Expr, FnDecl, HirId };
use rustc_hir::intravisit::{NestedVisitorMap, FnKind, walk_block, walk_expr, walk_fn, walk_crate, Visitor};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use crate::refactoring_invocation::TyContext;
use super::walk_desugars;

struct BlockCollector<'v> {
    tcx: TyCtxt<'v>,
    pos: Span,
    body_ids: Vec<BodyId>,
    selected_block: Option<(&'v Block<'v>, BodyId)>
}

// fn trim_span(tcx: &TyContext, mut span: Span) -> Span {
//     let source = tcx.get_source(span);
//     if let Some(d) = source.find(|c| !char::is_whitespace(c)) {
//         span = span.with_lo(BytePos(span.lo().0 + d as u32));
//     }
//     if let Some(d) = source.rfind(|c| !char::is_whitespace(c)) {
//         let diff = source.len() - d - 1;
//         span = span.with_hi(BytePos(span.hi().0 - diff as u32));
//     }
//     span
// }

/**
 * Span should either contain an assignment expression where the right hand side is a block expression
 * or a single block expression.
 * The block expression should not be the body of a function, loop, etc.
 */
pub fn collect_innermost_contained_block<'v>(tcx: &'v TyContext, pos: Span) -> Option<(&'v Block<'v>, BodyId)> {
    let mut v = BlockCollector {
        tcx: tcx.0,
        pos: /*trim_span(tcx, pos)*/ pos,
        body_ids: vec![],
        selected_block: None
    };

    walk_crate(&mut v, tcx.0.hir().krate());

    v.selected_block
}

impl BlockCollector<'_> {
    fn selection_contains_span(&self, span: Span) -> bool {
        self.pos == span.shrink_to_lo() || self.pos.contains(span)
    }
}

impl<'v> Visitor<'v> for BlockCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.hir())
    }
    fn visit_fn(
        &mut self,
        fk: FnKind<'v>,
        fd: &'v FnDecl,
        b: BodyId,
        s: Span,
        id: HirId,
    ) {
        self.body_ids.push(b);
        walk_fn(self, fk, fd, b, s, id);
        self.body_ids.pop();
    }

    fn visit_block(&mut self, block: &'v Block) {
        if self.body_ids.len() > 0 && self.selection_contains_span(block.span) {
            self.selected_block = Some((block, *self.body_ids.last().unwrap()));
            return;
        }
        // if !block.span.contains(self.pos) && !block.span.from_expansion() {
        //     return;
        // }
        walk_block(self, block);
    }
    fn visit_expr(&mut self, expr: &'v Expr) {
        if !walk_desugars(self, expr) {
            walk_expr(self, expr);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::run_ty_query;
    use crate::refactoring_invocation::{QueryResult, RefactoringErrorInternal};
    
    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<String> + Send> {
        Box::new(move |ty| {
            collect_innermost_contained_block(ty, ty.source().map_span(&file_name, from, to)?)
                .map(|(block, _)| ty.get_source(block.span))
                .ok_or_else(|| RefactoringErrorInternal::int("unwrap()"))
        })
    }
    
    #[test]
    fn should_collect_block_1() {
        let input = r#"
        fn foo() { 
            /*START*/{ }/*END*/
        }"#;
        let expected = Ok("{ }".to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_while_expr() {
        let input = r#"
        fn foo() { 
            while /*START*/{true}/*END*/{ }
        }"#;
        let expected = Ok("{true}".to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_while_body() {
        let input = r#"
        fn foo() { 
            while {true}/*START*/{1;}/*END*/
        }"#;
        let expected = Ok("{1;}".to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_for_body() {
        let input = r#"
        fn foo() {
            let x = vec![1];
            /*START*/{
                for i in x.iter().rev() {i;}
            };/*END*/
        }"#;
        let expected = Ok(r"{
                for i in x.iter().rev() {i;}
            }".to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_inside_while_let() {
        let input = r#"
        fn foo() {
            let i = Some(1);
            while let Some(_) = i {
                let x = 1;
                /*START*/{
                    let y = 1;
                };/*END*/
            }
        }"#;
        let expected = Ok(r"{
                    let y = 1;
                }".to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_inside_loop() {
        let input = r#"
        fn foo() {
            loop {
                /*START*/{};/*END*/
            }
        }"#;
        let expected = Ok(r"{}".to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
}
