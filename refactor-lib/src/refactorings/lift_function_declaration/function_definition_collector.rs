use rustc_span::{BytePos, Span};
use rustc_hir::{Item, ItemKind, Mod, HirId};
use rustc_hir::intravisit::{NestedVisitorMap, Visitor, walk_crate, walk_item};
use rustc_middle::hir::map::Map;
use crate::refactoring_invocation::{QueryResult, TyContext};

/// Collects a function declaration + closest parent module
pub fn collect_function_definition<'a, 'v>(tcx: &'a TyContext<'v>, pos: Span) -> QueryResult<FnDecl2<'v>> {
    let mut v = FnDefCollector {
        tcx,
        pos,
        fn_decl: None
    };

    walk_crate(&mut v, tcx.0.hir().krate());

    v.fn_decl.ok_or_else(|| tcx.source().span_err(pos, false))
}

impl<'a, 'v> Visitor<'v> for FnDefCollector<'a, 'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.0.hir())
    }
    fn visit_item(&mut self, i: &'v Item<'v>) {

        match &i.kind {
            ItemKind::Fn(..) => {
                if i.span == self.pos {
                    let parent_mod_id = self.tcx.0.parent_module(i.hir_id);
                    let (parent_mod, span, ..) = self.tcx.0.hir().get_module(parent_mod_id.to_def_id());

                    self.fn_decl = Some(FnDecl2 {
                        hir_id: i.hir_id,
                        span: i.span,
                        parent_mod,
                        mod_span: span
                    });
                }
            },
            _ => {}
        }

        walk_item(self, i);
    }
}
pub struct FnDecl2<'v> {
    pub span: Span,
    pub hir_id: HirId,
    pub parent_mod: &'v Mod<'v>,
    pub mod_span: Span
}
impl FnDecl2<'_> {
    /// Mod.inner contains first token past '{' but until '}' in some cases, so we have to shrink the span for those 
    /// see: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/struct.Mod.html
    pub fn get_parent_mod_inner(&self) -> Span {
        if self.mod_span.lo() != self.parent_mod.inner.lo() && self.mod_span.hi() == self.parent_mod.inner.hi() {
            self.parent_mod.inner.with_hi(BytePos(self.parent_mod.inner.hi().0 - 1))
        } else {
            self.parent_mod.inner
        }
    }
}

struct FnDefCollector<'a, 'v> {
    tcx: &'a TyContext<'v>,
    pos: Span,
    fn_decl: Option<FnDecl2<'v>>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::run_ty_query;

    #[derive(Debug, PartialEq)]
    struct FnDecl2Test {
        span: String,
        parent_mod_span: String
    }

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<FnDecl2Test> + Send> {
        Box::new(move |ty| {
            let span = ty.source().map_span(&file_name, from, to)?;
            let closure = collect_function_definition(ty, span)?;

            Ok(FnDecl2Test {
                span: ty.get_source(closure.span),
                parent_mod_span: ty.get_source(closure.get_parent_mod_inner()),
            })
        })
    }
    #[test]
    fn fn_decl_1() {
        let input = r#"
        fn main () {
            /*START*/fn foo() {}/*END*/;
        }"#;

        let expected = Ok(FnDecl2Test{
            span: "fn foo() {}".to_owned(),
            parent_mod_span: input.trim().to_string()
        });

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn fn_decl_2() {
        let input = r#"
        mod m1 {
            mod m2 {
                fn baz() {}
                fn main () {
                    /*START*/fn foo() {}/*END*/;
                }
            }
        }"#;

        let expected = Ok(FnDecl2Test{
            span: "fn foo() {}".to_owned(),
            parent_mod_span: r#"fn baz() {}
                fn main () {
                    /*START*/fn foo() {}/*END*/;
                }
            "#.to_owned()
        });

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
}