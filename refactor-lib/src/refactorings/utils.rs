use crate::change::Change;
use crate::refactor_definition::{RefactoringError, SourceCodeRange};
use rustc::ty::TyCtxt;
use rustc_hir::{HirId, StructField};
use rustc_span::{BytePos, FileName, Span};
use rustc_span::source_map::SourceMap;
use std::path::PathBuf;

pub fn get_file_offset(source_map: &SourceMap, file_name: &str) -> u32 {
    let file_name = FileName::Real(PathBuf::from(file_name.to_string()));
    let source_file = source_map.get_source_file(&file_name).unwrap();
    source_file.start_pos.0 as u32
}
fn get_filename(source_map: &SourceMap, span: Span) -> String {
    let filename = source_map.span_to_filename(span);
    if let FileName::Real(pathbuf) = &filename {
        if let Some(s) = pathbuf.to_str() {
            return s.to_string();
        }
    }
    panic!("unexpected file type: {:?}", filename);
}

pub fn map_range_to_span(source_map: &SourceMap, range: &SourceCodeRange) -> Result<Span, RefactoringError> {
    let filename = FileName::Real(std::path::PathBuf::from(&range.file_name));
    if let Some(source_file) = source_map.get_source_file(&filename) {
        Ok(Span::with_root_ctxt(
            BytePos(range.from + source_file.start_pos.0),
            BytePos(range.to + source_file.start_pos.0),
        ))
    } else {
        Err(RefactoringError::file_not_found(&range.file_name))
    }
}

pub fn map_change_from_span(source_map: &SourceMap, span: Span, replacement: String) -> Change {
    let filename = get_filename(source_map, span);
    let file_offset = get_file_offset(source_map, &filename);
    Change {
        file_name: filename,
        file_start_pos: file_offset,
        start: span.lo().0 - file_offset,
        end: span.hi().0 - file_offset,
        replacement,
    }
}

pub fn get_source(tcx: TyCtxt, span: Span) -> String {
    tcx.sess.source_map().span_to_snippet(span).unwrap()
}

pub fn get_struct_hir_id(tcx: TyCtxt<'_>, field: &StructField) -> HirId {
    let struct_def_id = field.hir_id.owner_def_id();
    tcx.hir().as_local_hir_id(struct_def_id).unwrap()
}