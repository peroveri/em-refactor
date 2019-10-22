use rustc::hir::{self, intravisit};
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

    intravisit::walk_crate(&mut v, tcx.hir().krate());

    v.field
}

impl<'v> intravisit::Visitor<'v> for FieldCollector<'v> {
    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'v> {
        intravisit::NestedVisitorMap::All(&self.tcx.hir())
    }

    fn visit_struct_field(&mut self, s: &'v hir::StructField) {
        if s.ident.span.eq(&self.span) {
            self.field = Some(s);
        }
        eprintln!("{:?}", s.span);
    }
}
