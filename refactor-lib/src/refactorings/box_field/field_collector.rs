use rustc::hir::{
    self,
    intravisit::{NestedVisitorMap, Visitor, walk_crate},
};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

/**
 * Given a selection the field of a struct where the identifier has the same span as 'span'
 */
struct FieldCollector<'v> {
    tcx: TyCtxt<'v>,
    span: Span,
    field: Option<&'v hir::StructField>,
}

pub fn collect_field(tcx: TyCtxt, span: Span) -> Option<(&hir::StructField)> {
    let mut v = FieldCollector {
        tcx,
        span,
        field: None,
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.field
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
    fn struct_field_collector_should_collect_field_declaration() {
        run_test(create_program(), |tcx| {
            let field = collect_field(tcx, create_test_span(56, 61));

            assert!(field.is_some());
            let field = field.unwrap();

            assert_eq!("field", field.ident.as_str().to_string());
        });
    }
}
