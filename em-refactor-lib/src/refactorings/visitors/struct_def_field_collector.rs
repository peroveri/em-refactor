use rustc_hir::intravisit::{walk_crate, walk_item, NestedVisitorMap, Visitor};
use rustc_hir::{Item, ItemKind, StructField};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

///
/// Returns the corresponding `StructField` in a struct definition if the `span` is equal to the `StructField`'s `span`
///
/// # Example
/// Given the program:
///
/// ```example
/// struct S {foo: u32}
///           | |
///           x y
/// ```
/// then `collect_field(x, y)` would return the `StructField` of `foo`
///
/// # Grammar
/// ```example
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
    use crate::test_utils::run_ty_query;
    use crate::refactoring_invocation::{QueryResult, TyContext};

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<Option<(String, String, usize)>> + Send> {
        Box::new(move |ty| {
            let span = ty.source().map_span(&file_name, from, to)?;

            if let Some((field, index)) = collect_field(ty.0, span) {
                Ok(Some((field.ident.as_str().to_string(), ty.get_source(field.span), index)))
            } else {
                Ok(None)
            }
        })
    }

    #[test]
    fn should_collect_field_definition() {
        let input = r#"
            struct T {not_this: i32}
            struct S {not_this: i32, /*START*/field/*END*/: u32}"#;
        
        let expected = Ok(Some((
            "field".to_owned(), "field/*END*/: u32".to_owned(), 1)));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_tuple_definition() {
        let input = r#"
            struct T (i32);
            struct S (i32, /*START*/u32/*END*/);"#;
        
        let expected = Ok(Some((
            "1".to_owned(), "u32".to_owned(), 1)));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
}
