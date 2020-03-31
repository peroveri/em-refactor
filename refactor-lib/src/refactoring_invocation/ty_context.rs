use rustc::ty::TyCtxt;
use rustc_span::Span;
use super::{RefactoringErrorInternal, SourceCodeRange};
pub struct TyContext<'a>(pub TyCtxt<'a>);

impl<'a> TyContext<'a> {
    pub fn new(ty: TyCtxt<'a>) -> Self {
        Self(ty)
    }
    pub fn map_range_to_span(&self, range: &SourceCodeRange) -> Result<Span, RefactoringErrorInternal> {
        crate::refactorings::utils::map_range_to_span(self.0.sess.source_map(), range)
    }
}
