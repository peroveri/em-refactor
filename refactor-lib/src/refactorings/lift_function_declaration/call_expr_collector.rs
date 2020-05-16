use rustc_hir::{Block, HirId, Expr, ExprKind, def_id::DefId, def::Res};
use rustc_hir::intravisit::{NestedVisitorMap, Visitor, walk_crate, walk_expr};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use crate::refactoring_invocation::TyContext;

// Def, decl or both?
pub fn collect_call_exprs<'v>(tcx: &'v TyContext, def_id: HirId) -> Vec<CallExpr<'v>> {
    let mut v = CallExprCollector {
        tcx: tcx.0,
        def_id: tcx.0.hir().local_def_id(def_id),
        calls: vec![]
    };

    walk_crate(&mut v, tcx.0.hir().krate());

    v.calls
}

impl<'v> CallExprCollector<'v> {
    fn visit_call_expr(&mut self, call_expr: &'v Expr<'v>, path_expr: &'v Expr<'v>, arg_exprs: &'v [Expr<'v>]) {

        match &path_expr.kind {
            ExprKind::Path(qpath) => {
                let typeck_table = self.tcx.typeck_tables_of(call_expr.hir_id.owner);
                match typeck_table.qpath_res(qpath, path_expr.hir_id) {
                    Res::Def(.., def_id) => {
                        if def_id == self.def_id {
                            self.calls.push(CallExpr {
                                arg_exprs,
                                call_expr,
                                path_expr
                            });
                        }
                    },
                    _ => {}
                }
            },
            ExprKind::Tup([expr]) => { // Tuple with single entry
                self.visit_call_expr(call_expr, expr, arg_exprs);
            },
            ExprKind::Block( Block { // Block with only an expression
                stmts: [], expr: Some(expr), ..
            }, ..) => {
                self.visit_call_expr(call_expr, expr, arg_exprs);
            },
            _ => {}
        }
    }
}

impl<'v> Visitor<'v> for CallExprCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.hir())
    }
    fn visit_expr(&mut self, expr: &'v Expr<'v>) {

        match expr.kind {
            ExprKind::Call(path_expr, arg_exprs) =>  {
                self.visit_call_expr(expr, path_expr, arg_exprs);
            },
            _ => { }
        }

        walk_expr(self, expr);
    }
}
struct CallExprCollector<'v> {
    tcx: TyCtxt<'v>,
    calls: Vec<CallExpr<'v>>,
    def_id: DefId
}
pub struct CallExpr<'v> {
    pub call_expr: &'v Expr<'v>,
    pub path_expr: &'v Expr<'v>,
    pub arg_exprs: &'v [Expr<'v>],
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::run_ty_query;
    use crate::refactoring_invocation::QueryResult;
    use super::super::function_definition_collector::collect_function_definition;

    #[derive(Debug, PartialEq)]
    struct FnDecl2Test {
        call: String,
        args: String,
        path: String,
    }

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Vec<FnDecl2Test>> + Send> {
        Box::new(move |ty| {
            let span = ty.source().map_span(&file_name, from, to)?;
            let fn_def = collect_function_definition(ty, span)?;
            let call_exprs = collect_call_exprs(ty, fn_def.hir_id);

            Ok(call_exprs.iter().map(|call| FnDecl2Test{
                call: ty.get_source(call.call_expr.span),
                args: call.arg_exprs.iter().map(|expr| ty.get_source(expr.span)).collect::<Vec<_>>().join(", "),
                path: ty.get_source(call.path_expr.span),
            }).collect())
        })
    }
    #[test]
    fn fn_decl_1() {
        let input = r#"
        fn main () {
            /*START*/fn foo() {}/*END*/;
        }"#;

        let expected = Ok(vec![]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn fn_decl_2() {
        let input = r#"
        fn main () {
            /*START*/fn foo() {}/*END*/;
            foo();
        }"#;

        let expected = Ok(vec![
            FnDecl2Test {
                call: "foo()".to_owned(),
                args: "".to_owned(),
                path: "foo".to_owned(),
            }
        ]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn fn_decl_3() {
        let input = r#"
        fn main () {
            /*START*/fn foo(_: i32) {}/*END*/;
            foo(1);
            foo(2);
        }"#;

        let expected = Ok(vec![
            FnDecl2Test {
                call: "foo(1)".to_owned(),
                args: "1".to_owned(),
                path: "foo".to_owned(),
            },
            FnDecl2Test {
                call: "foo(2)".to_owned(),
                args: "2".to_owned(),
                path: "foo".to_owned(),
            }
        ]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn fn_decl_4() {
        let input = r#"
        fn main () {
            /*START*/fn foo(_: i32) {}/*END*/;
            ({foo})(1);
        }"#;

        let expected = Ok(vec![
            FnDecl2Test {
                call: "({foo})(1)".to_owned(),
                args: "1".to_owned(),
                path: "foo".to_owned(),
            },
        ]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
}