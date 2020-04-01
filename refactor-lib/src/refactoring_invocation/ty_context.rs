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
    #[cfg(test)]
    pub fn get_span(&self, file_name: &str, from: u32, to: u32) -> Result<Span, RefactoringErrorInternal> {
        let file_name = file_name.to_string();
        crate::refactorings::utils::map_range_to_span(self.0.sess.source_map(), &SourceCodeRange {file_name, from, to})
    }
}
