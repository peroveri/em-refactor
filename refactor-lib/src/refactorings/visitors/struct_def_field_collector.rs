use rustc_hir::intravisit::{walk_crate, walk_item, NestedVisitorMap, Visitor};
use rustc_hir::{Item, ItemKind, StructField};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;

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
pub fn collect_field(tcx: TyCtxt, span: Span) -> Option<(&StructField, usize)> {
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
    field: Option<(&'v StructField<'v>, usize)>,
}

impl<'v> Visitor<'v> for FieldCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.hir())
    }
    fn visit_item(&mut self, item: &'v Item<'v>) {
        if let ItemKind::Struct(data, _) = &item.kind {
            for (i, field) in data.fields().iter().enumerate() {
                if field.ident.span.contains(self.span) {
                    self.field = Some((field, i));
                }
            }
        }
        walk_item(self, item);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::refactorings::utils::get_source;
    use crate::{create_test_span, run_after_analysis};
    use quote::quote;

    fn create_struct() -> quote::__rt::TokenStream {
        quote! {
            struct T {not_this: i32}
            struct S {not_this: i32, field: u32}
        }
    }
    fn create_tuple() -> quote::__rt::TokenStream {
        quote! {
            struct T ( i32 );
            struct S ( i32, u32 );
        }
    }

    #[test]
    fn struct_field_collector_should_collect_field_definition() {
        run_after_analysis(create_struct(), |tcx| {
            let field = collect_field(tcx, create_test_span(56, 61));

            assert!(field.is_some());
            let (field, _) = field.unwrap();

            assert_eq!("field", field.ident.as_str().to_string());
        });
    }
    #[test]
    fn struct_field_collector_should_collect_tuple_definition() {
        run_after_analysis(create_tuple(), |tcx| {
            let field = collect_field(tcx, create_test_span(36, 39));
            assert!(field.is_some());
            let (field, index) = field.unwrap();

            assert_eq!(1, index);
            assert_eq!("1", field.ident.as_str().to_string());
            assert_eq!("u32", get_source(tcx, field.span));
            assert!(field.is_positional());
        });
    }
}
