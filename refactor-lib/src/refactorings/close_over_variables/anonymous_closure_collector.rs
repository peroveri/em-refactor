use rustc::ty::TyCtxt;
use rustc_span::{BytePos, Span};
use rustc_hir::{BodyId, Expr, ExprKind};
use rustc_hir::intravisit::{NestedVisitorMap, Visitor, walk_expr, walk_crate};
use rustc::hir::map::Map;

pub fn collect_anonymous_closure<'v>(tcx: TyCtxt<'v>, pos: Span) -> Option<Closure> {
    let mut v = ClosureCollector {
        tcx,
        pos,
        result: None
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.result
}

impl<'v> Visitor<'v> for ClosureCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, Self::Map> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_expr(&mut self, ex: &'v Expr<'v>) {

        match ex.kind {
            ExprKind::Call(expr, args) => {
                if self.pos.eq(&ex.span) {
                    let arg_span = ex.span.with_hi(BytePos(ex.span.hi().0 - 1)).shrink_to_hi();

                    match expr.kind {
                        ExprKind::Closure(_capture, _fn_decl, _body, _span, _movability) => {
                            self.result = Some(Closure {
                                args: arg_span,
                                body_id: _body,
                                body_span: ex.span, // TODO
                                has_params: args.len() > 0,
                                params: _span.with_hi(BytePos(_span.hi().0 - 1)).shrink_to_hi()
                            });
                            return;
                        },
                        _ => {}
                    }
                }

            },
            _ => {}
        }

        walk_expr(self, ex);
    }
}
pub struct Closure {
    pub args: Span,
    pub params: Span,
    pub has_params: bool,
    pub body_id: BodyId,
    pub body_span: Span
}

struct ClosureCollector<'v> {
    tcx: TyCtxt<'v>,
    pos: Span,
    result: Option<Closure>
}

#[cfg(test)]
mod test {
    use super::*;
    use quote::quote;
    use crate::{create_test_span, run_after_analysis};
    use crate::refactorings::utils::get_source;

    #[test]
    fn collect_anonymous_closure_foo() {
        run_after_analysis(quote! {
            fn main ( ) { ( | | { } ) ( ) ; }
        }, |tcx| {
            let closure_span = create_test_span(14, 29);
            let params = create_test_span(18, 18);
            let args = create_test_span(28, 28);
            let closure = collect_anonymous_closure(tcx, closure_span);
            if !closure.is_some() {
                panic!(get_source(tcx, closure_span));
            }

            let closure = closure.unwrap();
            assert!(!closure.has_params);
            assert_eq!(params, closure.params);
            assert_eq!(args, closure.args);
        });
    }
    #[test]
    fn collect_anonymous_closure_foo2() {
        run_after_analysis(quote! {
            fn main ( ) { ( | i : _ | { } ) ( 0 ) ; }
        }, |tcx| {
            let closure_span = create_test_span(14, 37);
            let params = create_test_span(24, 24);
            let args = create_test_span(36, 36);
            let closure = collect_anonymous_closure(tcx, closure_span);
            if !closure.is_some() {
                panic!(get_source(tcx, closure_span));
            }

            let closure = closure.unwrap();
            assert!(closure.has_params);
            assert_eq!(params, closure.params);
            assert_eq!(args, closure.args);
        });
    }
}