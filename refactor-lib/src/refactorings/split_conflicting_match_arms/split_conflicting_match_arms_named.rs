use rustc::ty::TyCtxt;
use rustc_hir::HirId;
use rustc_span::Span;
use crate::output_types::FileStringReplacement;
use crate::refactoring_invocation::RefactoringErrorInternal;

pub fn do_refactoring(tcx: TyCtxt, struct_hir_id: HirId, field_ident: &str, field_ty_span: Span) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {

    panic!()
}