use rustc_hir::{Body, BodyId, HirId, QPath, def_id::DefId};
use rustc_hir::intravisit::{NestedVisitorMap, Visitor, walk_body, walk_crate};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use crate::refactoring_invocation::TyContext;

pub fn collect_qpaths<'v>(tcx: &'v TyContext, def_id: DefId) -> Vec<Span> {
    let mut v = CallExprCollector {
        tcx: tcx.0,
        def_id,
        paths: vec![],
        bodies: vec![]
    };

    walk_crate(&mut v, tcx.0.hir().krate());

    v.paths
}

impl<'v> Visitor<'v> for CallExprCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.hir())
    }
    fn visit_body(&mut self, b: &'v Body<'v>) {
        self.bodies.push(b.id());
        walk_body(self, b);
        self.bodies.pop();
    }
    fn visit_qpath(&mut self, qpath: &'v QPath<'v>, id: HirId, span: Span) {
        if let Some(body_id) = self.bodies.last() {
            let typeck_table = self.tcx.typeck_tables_of(self.tcx.hir().body_owner_def_id(*body_id));
            if let Some(def_id) = typeck_table.qpath_res(qpath, id).opt_def_id() {
                if def_id == self.def_id {
                    self.paths.push(span);
                }
            }
        }
    }
}
struct CallExprCollector<'v> {
    tcx: TyCtxt<'v>,
    paths: Vec<Span>,
    def_id: DefId,
    bodies: Vec<BodyId>
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::run_ty_query;
    use crate::refactoring_invocation::QueryResult;
    use crate::refactorings::visitors::hir::collect_function_definition;

    #[derive(Debug, PartialEq)]
    struct FnDecl2Test {
        call: String,
        args: String,
        path: String,
        path_val: String,
    }

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Vec<String>> + Send> {
        Box::new(move |ty| {
            let span = ty.source().map_span(&file_name, from, to)?;
            let fn_def = collect_function_definition(ty, span)?;
            Ok(
                collect_qpaths(ty, fn_def.hir_id)
                    .iter()
                    .map(|span| ty.get_source(*span))
                    .collect()
            )
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
            "foo".to_owned()
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
            "foo".to_owned(),
            "foo".to_owned()
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
            "foo".to_owned()
        ]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn fn_decl_5() {
        let input = r#"
        struct S;
        trait T {fn baz() {}}
        impl T for S {
            fn baz() {
                /*START*/fn foo() {}/*END*/
                ({foo})();
            }
        }
        "#;

        let expected = Ok(vec![
            "foo".to_owned()
        ]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn fn_decl_6() {
        let input = r#"
        struct S;
        mod m1 {
            /*START*/pub fn foo() {}/*END*/
        }
        fn baz() {
            m1::foo();
        }"#;

        let expected = Ok(vec![
            "m1::foo".to_owned()
        ]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn fn_decl_7() {
        let input = r#"
        /*START*/pub fn foo() {}/*END*/
        fn baz() {
            let z = foo;
            z();
        }"#;

        let expected = Ok(vec![
            "foo".to_owned()
        ]);

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
}