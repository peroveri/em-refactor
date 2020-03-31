use rustc::ty::TyCtxt;
use rustc_hir::HirId;
use rustc_span::Span;
use crate::refactoring_invocation::{AstDiff, QueryResult};

pub fn do_refactoring(_tcx: TyCtxt, _struct_hir_id: HirId, _field_ident: &str, _field_ty_span: Span) -> QueryResult<AstDiff> {

    panic!()
}