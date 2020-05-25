use rustc_hir::BodyId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use em_refactor_lib_types::FileStringReplacement;
use crate::refactorings::utils::map_change_from_span;
use super::{QueryResult, SourceMapContext};
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
        self.source().get_source(span)
    }
    pub fn map_change(&self, span: Span, replacement: String) -> QueryResult<FileStringReplacement> {
        map_change_from_span(self.0.sess.source_map(), span, replacement)
    }
    pub fn get_body_span(&self, body_id: BodyId) -> Span {
        self.0.hir().body(body_id).value.span
    }
}
