use crate::change::Change;
use crate::refactor_definition::SourceCodeRange;
use rustc::ty::TyCtxt;
use std::path::PathBuf;
use syntax::source_map::FileName;
use syntax_pos::{BytePos, Span};

pub fn get_file_offset(tcx: &TyCtxt, file_name: &str) -> u32 {
    let file_name = FileName::Real(PathBuf::from(file_name.to_string()));
    let source_file = tcx.sess.source_map().get_source_file(&file_name).unwrap();
    source_file.start_pos.0 as u32
}

pub fn map_to_span(
    source_map: &syntax::source_map::SourceMap,
    selection: (u32, u32),
    file: &str,
) -> Span {
    let filename = FileName::Real(std::path::PathBuf::from(file));
    let source_file = source_map.get_source_file(&filename).unwrap();
    Span::with_root_ctxt(
        BytePos(selection.0 + source_file.start_pos.0),
        BytePos(selection.1 + source_file.start_pos.0),
    )
}

pub fn map_range_to_span(tcx: TyCtxt, range: &SourceCodeRange) -> Span {
    map_to_span(
        tcx.sess.source_map(),
        (range.from, range.to),
        &range.file_name,
    )
}

pub fn map_change(tcx: TyCtxt, range: &SourceCodeRange, replacement: String) -> Change {
    Change {
        file_name: range.file_name.to_string(),
        file_start_pos: get_file_offset(&tcx, &range.file_name),
        start: range.from,
        end: range.to,
        replacement,
    }
}

pub fn map_change_from_span(
    tcx: &TyCtxt,
    span: Span,
    file_name: &str,
    replacement: String,
) -> Change {
    let file_offset = get_file_offset(tcx, &file_name);
    Change {
        file_name: file_name.to_string(),
        file_start_pos: file_offset,
        start: span.lo().0 - file_offset,
        end: span.hi().0 - file_offset,
        replacement,
    }
}

pub fn get_source(tcx: &TyCtxt, span: Span) -> String {
    tcx.sess.source_map().span_to_snippet(span).unwrap()
}
