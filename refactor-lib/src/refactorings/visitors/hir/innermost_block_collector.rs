use rustc_hir::{BodyId, Block, FnDecl, HirId};
use rustc_hir::intravisit::{NestedVisitorMap, Visitor, FnKind, walk_fn, walk_block, walk_crate};
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
        Err(tcx.source().span_err(pos))
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
        if let Some(expr) = &body.expr {
            walk_desugars(self, &expr.kind);
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
}

#[cfg(test)]
mod test {
    use quote::quote;
    use super::test_util::{assert_fail, assert_success};
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
        let expected = Err(RefactoringErrorInternal::invalid_selection_with_code(9, 9, ""));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }    
    #[test]
    fn innermost_block_collector_fn_with_single_block() {
        assert_success(quote! {
            fn f ( ) { 1 ; }
        }, (10, 15), "{ 1 ; }");
    }
    #[test]
    fn innermost_block_collector_desugar_for() {
        let p = quote!{
            fn f ( ) { for i in { vec ! [ 1 ] } { i ; } }
        };
        assert_success(p.clone(), (10, 44), "{ for i in { vec ! [ 1 ] } { i ; } }");
        assert_success(p.clone(), (21, 34), "{ vec ! [ 1 ] }");
        assert_success(p.clone(), (37, 42), "{ i ; }");
    }
    #[test]
    fn innermost_block_collector_desugar_if() {
        let p = quote! {
            fn f ( ) { if { true } { 1 ; } }
        };
        assert_success(p.clone(), (10, 31), "{ if { true } { 1 ; } }");
        assert_success(p.clone(), (15, 21), "{ true }");
        assert_success(p.clone(), (24, 29), "{ 1 ; }");
    }
    #[test]
    fn innermost_block_collector_desugar_ifelse() {
        let p = quote! {
            fn f ( ) { if { true } { 1 ; } else { 2 ; } }
        };
        assert_success(p.clone(), (10, 44), "{ if { true } { 1 ; } else { 2 ; } }");
        assert_success(p.clone(), (15, 21), "{ true }");
        assert_success(p.clone(), (24, 29), "{ 1 ; }");
        assert_success(p.clone(), (37, 42), "{ 2 ; }");
    }
    #[test]
    fn innermost_block_collector_desugar_iflet() {
        let p = quote! {
            fn f ( ) { if let _ = { true } { 1 ; } }
        };
        assert_success(p.clone(), (10, 39), "{ if let _ = { true } { 1 ; } }");
        assert_success(p.clone(), (23, 29), "{ true }");
        assert_success(p.clone(), (32, 37), "{ 1 ; }");
    }
    #[test]
    fn innermost_block_collector_desugar_ifletelse() {
        let p = quote! {
            fn f ( ) { if let _ = { true } { 1 ; } else { 2 ; } }
        };
        assert_success(p.clone(), (10, 52), "{ if let _ = { true } { 1 ; } else { 2 ; } }");
        assert_success(p.clone(), (23, 29), "{ true }");
        assert_success(p.clone(), (32, 37), "{ 1 ; }");
        assert_success(p.clone(), (45, 50), "{ 2 ; }");
    }
    #[test]
    fn innermost_block_collector_desugar_while() {
        let p = quote! {
            fn f ( ) { while { true } { 1 ; } }
        };
        assert_success(p.clone(), (10, 34), "{ while { true } { 1 ; } }");
        assert_success(p.clone(), (18, 24), "{ true }");
        assert_success(p.clone(), (27, 32), "{ 1 ; }");
    }
    #[test]
    fn innermost_block_collector_desugar_whilelet() {
        let p = quote! {
            fn f ( ) { while let _ = { true } { 1 ; } }
        };
        assert_success(p.clone(), (10, 42), "{ while let _ = { true } { 1 ; } }");
        assert_success(p.clone(), (26, 32), "{ true }");
        assert_success(p.clone(), (35, 40), "{ 1 ; }");
    }
    // TODO: try + await
    #[test]
    fn innermost_block_collector_const() {
        assert_fail( quote! {
            const _ : i32 = { 0 } ;
        }, (17, 20));
    }
}

#[cfg(test)]
mod test_util {
    use super::*;
    use quote::__private::TokenStream;
    use crate::{create_test_span, run_after_analysis};
    use crate::refactorings::utils::get_source;

    pub fn assert_success(prog: TokenStream, span: (u32, u32), expected: &str) {
        run_after_analysis(prog, |tcx| {
            let tcx1 = TyContext(tcx);
            let (block, _) = collect_innermost_block(&tcx1, create_test_span(span.0, span.1)).unwrap();
            
            assert_eq!(get_source(tcx, block.span), expected);
        });
    }
    pub fn assert_fail(prog: TokenStream, span: (u32, u32)) {
        run_after_analysis(prog, |tcx| {
            let tcx1 = TyContext(tcx);
            assert!(collect_innermost_block(&tcx1, create_test_span(span.0, span.1)).is_err());
        });
    }
}