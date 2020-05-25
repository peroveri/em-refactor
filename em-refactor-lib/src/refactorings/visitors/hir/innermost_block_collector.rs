use rustc_hir::{BodyId, Block, Expr, FnDecl, HirId};
use rustc_hir::intravisit::{NestedVisitorMap, Visitor, FnKind, walk_fn, walk_block, walk_crate, walk_expr};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use super::desugaring::walk_desugars;
use crate::refactoring_invocation::{QueryResult, TyContext};

struct BlockCollector<'v> {
    tcx: TyCtxt<'v>,
    pos: Span,
    body_id: Vec<BodyId>,
    result: Option<(&'v Block<'v>, BodyId)>
}

/**
 * Given a selection (byte start, byte end) and file name, this visitor finds
 * the innermost block containing `pos`
 */
pub fn collect_innermost_block<'v>(tcx: &'v TyContext, pos: Span) -> QueryResult<(&'v Block<'v>, BodyId)> {
    let mut v = BlockCollector {
        tcx: tcx.0,
        pos,
        body_id: vec![],
        result: None
    };

    walk_crate(&mut v, tcx.0.hir().krate());

    if let Some(r) = v.result {
        Ok(r)
    } else {
        Err(tcx.source().span_err(pos, false))
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
        self.body_id.push(b);
        walk_fn(self, fk, fd, b, s, id);
        self.body_id.pop();
    }

    fn visit_block(&mut self, body: &'v Block) {
        if self.body_id.is_empty() {
            return;
        }
        if !body.span.contains(self.pos) {
            return;
        }

        walk_block(self, body);
        if self.result.is_some() {
            return;
        }

        self.result = Some((
             body,
            *self.body_id.last().unwrap()));
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
    use crate::refactoring_invocation::RefactoringErrorInternal;
    
    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<String> + Send> {
        Box::new(move |ty| {
            collect_innermost_block(ty, ty.source().map_span(&file_name, from, to)?)
                .map(|(block, _)| ty.get_source(block.span))
        })
    }

    #[test]
    fn invalid_selection() {
        let input = "/*START*//*END*/fn foo() { }";
        let expected = Err(RefactoringErrorInternal::invalid_selection_with_code(9, 9, "", false));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }    
    #[test]
    fn fn_with_single_block() {
        let input = r#"
        fn foo() { 
            /*START*/1;/*END*/
        }"#;

        let expected = Ok(r#"{ 
            /*START*/1;/*END*/
        }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn desugar_for() {
        let input = r#"
        fn foo() {
            for i in { vec![ 1 ] } {
                /*START*/print!("{}", i);/*END*/
                return;
            }
        }"#;

        let expected = Ok(r#"{
                /*START*/print!("{}", i);/*END*/
                return;
            }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn desugar_for_outside() {
        let input = r#"
        fn foo() {
            /*START*/for i in { vec![ 1 ] } {
                print!("{}", i);
                return;
            }/*END*/
        }"#;

        let expected = Ok(r#"{
            /*START*/for i in { vec![ 1 ] } {
                print!("{}", i);
                return;
            }/*END*/
        }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn desugar_if() {
        let input = r#"
        fn foo(c: bool) {
            if c {
                /*START*/print!("{}", c);/*END*/
            }
        }"#;

        let expected = Ok(r#"{
                /*START*/print!("{}", c);/*END*/
            }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn desugar_ifelse_if() {
        let input = r#"
        fn foo(c: bool) {
            if c {
                /*START*/print!("{}", c);/*END*/
            } else {
                print!("else: {}", c);
            }
        }"#;

        let expected = Ok(r#"{
                /*START*/print!("{}", c);/*END*/
            }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn desugar_ifelse_else() {
        let input = r#"
        fn foo(c: bool) {
            if c {
                print!("{}", c);
            } else {
                /*START*/print!("else: {}", c);/*END*/
            }
        }"#;

        let expected = Ok(r#"{
                /*START*/print!("else: {}", c);/*END*/
            }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn desugar_iflet() {
        let input = r#"
        fn foo(c: Option<i32>) {
            if let Some(i) = c {
                /*START*/print!("{}", i);/*END*/
            }
        }"#;

        let expected = Ok(r#"{
                /*START*/print!("{}", i);/*END*/
            }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn desugar_ifletelse_if() {
        let input = r#"
        fn foo(c: Option<i32>) {
            if let Some(i) = c {
                /*START*/print!("{}", i);/*END*/
            } else {
                print!("{:?}", c);
            }
        }"#;

        let expected = Ok(r#"{
                /*START*/print!("{}", i);/*END*/
            }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn desugar_ifletelse_else() {
        let input = r#"
        fn foo(c: Option<i32>) {
            if let Some(i) = c {
                print!("{}", i);
            } else {
                /*START*/print!("{:?}", c);/*END*/
            }
        }"#;

        let expected = Ok(r#"{
                /*START*/print!("{:?}", c);/*END*/
            }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn desugar_while() {
        let input = r#"
        fn foo(mut i: i32) {
            while i > 0 {
                /*START*/print!("{}", i);/*END*/
                i += 1;
            }
        }"#;

        let expected = Ok(r#"{
                /*START*/print!("{}", i);/*END*/
                i += 1;
            }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    #[ignore]
    fn desugar_whilelet() {
        let input = r#"
        fn foo(mut i: Option<i32>) {
            while let Some(x) = i {
                /*START*/print!("{}", x);/*END*/
                i = None;
                return;
            }
        }"#;

        let expected = Ok(r#"{
                /*START*/print!("{}", x);/*END*/
                i = None;
                return;
            }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn desugar_whilelet_outside() {
        let input = r#"
        fn foo(mut i: Option<i32>) {
            /*START*/while let Some(x) = i {
                print!("{}", x);
                i = None;
                return;
            }/*END*/
        }"#;

        let expected = Ok(r#"{
            /*START*/while let Some(x) = i {
                print!("{}", x);
                i = None;
                return;
            }/*END*/
        }"#.to_owned());

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn shouldnt_collect_const() {
        let input = r#"const C: i32 = {
            /*START*/1/*END*/
        };"#;
        let expected = Err(RefactoringErrorInternal::invalid_selection_with_code(38, 39, "1", false));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    // TODO: try + await
}
