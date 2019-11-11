use rustc::hir::{
    self,
    intravisit::{walk_crate, NestedVisitorMap, Visitor},
};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

///
/// Returns the corresponding `StructField` in a struct definition if the `span` is equal to the `StructField`'s `span`
/// 
/// # Example
/// Given the program: 
/// 
/// ```
/// struct S {foo: u32}
///           | |
///           x y
/// ```
/// then `collect_field(x, y)` would return the `StructField` of `foo`
/// 
/// # Grammar
/// ```
/// StructStruct:
///   struct IDENTIFIER  Generics? WhereClause? ( { StructFields? } | ; )
/// StructFields :
///   StructField (, StructField)\* ,?
/// StructField :
///   OuterAttribute\* Visibility? IDENTIFIER : Type
/// ```
/// [Structs grammar](https://doc.rust-lang.org/stable/reference/items/structs.html)
/// 
/// TODO: Is is possible to query this directly in some way?
pub fn collect_field(tcx: TyCtxt, span: Span) -> Option<(&hir::StructField)> {
    let mut v = FieldCollector {
        tcx,
        span,
        field: None,
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.field
}

struct FieldCollector<'v> {
    tcx: TyCtxt<'v>,
    span: Span,
    field: Option<&'v hir::StructField>,
}

impl<'v> Visitor<'v> for FieldCollector<'v> {
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, 'v> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_struct_field(&mut self, s: &'v hir::StructField) {
        if s.ident.span.eq(&self.span) {
            self.field = Some(s);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{create_test_span, run_test};
    use quote::quote;

    fn create_program() -> quote::__rt::TokenStream {
        quote! {
            struct T {not_this: i32}
            struct S {not_this: i32, field: u32}
        }
    }

    #[test]
    fn struct_field_collector_should_collect_field_definition() {
        run_test(create_program(), |tcx| {
            let field = collect_field(tcx, create_test_span(56, 61));

            assert!(field.is_some());
            let field = field.unwrap();

            assert_eq!("field", field.ident.as_str().to_string());
        });
    }
}
