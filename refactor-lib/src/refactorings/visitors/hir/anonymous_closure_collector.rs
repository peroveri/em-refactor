use rustc_span::{BytePos, Span};
use rustc_hir::{BodyId, Expr, ExprKind, FnDecl};
use rustc_hir::intravisit::{NestedVisitorMap, Visitor, walk_expr, walk_crate};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use crate::refactoring_invocation::{QueryResult, RefactoringErrorInternal, TyContext};

pub fn collect_anonymous_closure<'v>(tcx: &'v TyContext, pos: Span) -> QueryResult<Closure<'v>> {
    let mut v = ClosureCollector {
        tcx: tcx.0,
        pos,
        result: None
    };

    walk_crate(&mut v, tcx.0.hir().krate());

    v.result.ok_or_else(|| RefactoringErrorInternal::invalid_selection_with_code(pos.lo().0, pos.hi().0, &tcx.get_source(pos)))
}

impl<'v> Visitor<'v> for ClosureCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.hir())
    }
    fn visit_expr(&mut self, ex: &'v Expr<'v>) {

        match ex.kind {
            ExprKind::Call(call_fn_expr, args) => {
                if self.pos.eq(&ex.span) {
                    match call_fn_expr.kind {
                        ExprKind::Closure(_capture, _fn_decl, _body, closure_params, _movability) => {
                            self.result = Some(Closure {
                                body_id: _body,
                                call_expr: ex,
                                call_fn_expr: call_fn_expr,
                                has_params: args.len() > 0,
                                params_span: closure_params,
                                fn_decl: _fn_decl,
                                args_1: args
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
pub struct Closure<'v> {
    pub params_span: Span,
    pub call_expr: &'v Expr<'v>,
    pub call_fn_expr: &'v Expr<'v>,
    pub has_params: bool,
    pub body_id: BodyId,
    pub fn_decl: &'v FnDecl<'v>,
    pub args_1: &'v [Expr<'v>]
}

fn get_arg_span(args: &[Expr], call_expr: &Expr) -> Span {
    if args.len() > 0 {
        args[0].span.with_hi(args[args.len()-1].span.hi())
    } else {
        call_expr.span.with_hi(BytePos(call_expr.span.hi().0 - 1)).shrink_to_hi()
    }
}

impl<'v> Closure<'v> {
    pub fn get_next_param_pos(&self) -> Span {
        self.params_span.with_hi(BytePos(self.params_span.hi().0 - 1)).shrink_to_hi()
    }
    pub fn get_next_arg_pos(&self) -> Span {
        let arg_span = get_arg_span(self.args_1, self.call_expr);
        arg_span.with_hi(BytePos(arg_span.hi().0 - 1)).shrink_to_hi()
    }
}

struct ClosureCollector<'v> {
    tcx: TyCtxt<'v>,
    pos: Span,
    result: Option<Closure<'v>>
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::assert_success3;

    #[derive(Debug, PartialEq)]
    struct Closure {
        args: String,
        params: String,
        has_params: bool,
        fn_inputs: String,
        body_params: String,
    }

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Closure> + Send> {
        Box::new(move |ty| {
            let span = ty.get_span(&file_name, from, to)?;
            let closure = collect_anonymous_closure(ty, span).unwrap();

            Ok(Closure {
                args: closure.args_1.iter().map(|e| ty.get_source(e.span)).collect::<Vec<_>>().join(", "),
                params: ty.get_source(closure.params_span),
                has_params: closure.has_params,
                fn_inputs: closure.fn_decl.inputs.iter().map(|t| ty.get_source(t.span)).collect::<Vec<_>>().join(", "),
                body_params: ty.0.hir().body(closure.body_id).params.iter().map(|t| ty.get_source(t.pat.span)).collect::<Vec<_>>().join(", "),
            })
        })
    }
    #[test]
    fn closure_without_params() {
        assert_success3(
r#"fn main () {
    /*START*/( || { } ) ( )/*END*/; 
}"#,
        map,
        Closure {
            args: "".to_owned(),
            params: "||".to_owned(),
            has_params: false,
            fn_inputs: "".to_owned(),
            body_params: "".to_owned(),
        });
    }
    #[test]
    fn closure_with_single_param() {
        assert_success3(
r#"fn main () {
    /*START*/( |i: _| { i; } ) (0)/*END*/; 
}"#,
        map,
        Closure {
            args: "0".to_owned(),
            params: "|i: _|".to_owned(),
            has_params: true,
            fn_inputs: "_".to_owned(),
            body_params: "i".to_owned(),
        });
    }
    #[test]
    fn closure_with_multiple_param() {
        assert_success3(
r#"fn main () {
    let mut x = 2;
    let z = 
    /*START*/( |a: _, b: &_, c: &mut _| { a } ) (0, &1, &mut x)/*END*/; 
}"#,
        map,
        Closure {
            args: "0, &1, &mut x".to_owned(),
            params: "|a: _, b: &_, c: &mut _|".to_owned(),
            has_params: true,
            fn_inputs: "_, &_, &mut _".to_owned(),
            body_params: "a, b, c".to_owned(),
        });
    }
}