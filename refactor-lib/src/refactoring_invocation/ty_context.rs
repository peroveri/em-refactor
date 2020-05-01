use rustc_hir::BodyId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::source_map::SourceMap;
use refactor_lib_types::FileStringReplacement;
use crate::refactorings::utils::map_change_from_span;
use super::{RefactoringErrorInternal, SourceCodeRange, SourceMapContext};
pub struct TyContext<'a>(pub TyCtxt<'a>);

impl<'a> TyContext<'a> {
    pub fn new(ty: TyCtxt<'a>) -> Self {
        Self(ty)
    }
    pub fn source(&self) -> SourceMapContext<'a> {
        SourceMapContext {
            source_map: self.0.sess.source_map()
        }
    }
    pub fn get_source(&self, span: Span) -> String {
        self.0.sess.source_map().span_to_snippet(span).unwrap()
    }
    pub fn get_source_map(&self) -> &SourceMap {
        self.0.sess.source_map()
    }
    #[cfg(test)]
    pub fn get_span(&self, file_name: &str, from: u32, to: u32) -> Result<Span, RefactoringErrorInternal> {
        let file_name = file_name.to_string();
        crate::refactorings::utils::map_range_to_span(self.0.sess.source_map(), &SourceCodeRange {file_name, from, to})
    }
    pub fn map_change(&self, span: Span, replacement: String) -> FileStringReplacement {
        map_change_from_span(self.0.sess.source_map(), span, replacement)
    }
    pub fn map_range_to_span(&self, range: &SourceCodeRange) -> Result<Span, RefactoringErrorInternal> {
        crate::refactorings::utils::map_range_to_span(self.0.sess.source_map(), range)
    }
    pub fn span_err(&self, span: Span) -> RefactoringErrorInternal {
        RefactoringErrorInternal::invalid_selection_with_code(span.lo().0, span.hi().0, &self.get_source(span))
    }
    pub fn get_body_span(&self, body_id: BodyId) -> Span {
        self.0.hir().body(body_id).value.span
    }
}
