use rustc_hir::{StructField, HirId};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use em_refactor_lib_types::FileStringReplacement;
use crate::refactorings::utils::map_change_from_span;
use super::{QueryResult, SourceMapContext};
pub struct TyContext<'a>(pub TyCtxt<'a>);

impl<'a> TyContext<'a> {
    pub(crate) fn new(ty: TyCtxt<'a>) -> Self {
        Self(ty)
    }
    pub(crate) fn source(&self) -> SourceMapContext<'a> {
        SourceMapContext {
            source_map: self.0.sess.source_map()
        }
    }
    pub(crate) fn get_source(&self, span: Span) -> String {
        self.source().get_source(span)
    }
    pub(crate) fn map_change(&self, span: Span, replacement: String) -> QueryResult<FileStringReplacement> {
        map_change_from_span(self.0.sess.source_map(), span, replacement)
    }

    pub(crate) fn get_struct_hir_id(&self, field: &StructField) -> HirId {
        let struct_def_id = field.hir_id.owner.to_def_id();
        self.0.hir().as_local_hir_id(struct_def_id).unwrap()
    }
}
