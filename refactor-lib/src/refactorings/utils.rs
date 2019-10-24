use crate::change::Change;
use crate::refactor_definition::SourceCodeRange;
use rustc::ty::TyCtxt;
use std::path::PathBuf;
use syntax::source_map::FileName;
use syntax_pos::{BytePos, Span};

pub fn get_file_offset(tcx: TyCtxt, file_name: &str) -> u32 {
    let file_name = FileName::Real(PathBuf::from(file_name.to_string()));
    let source_file = tcx.sess.source_map().get_source_file(&file_name).unwrap();
    source_file.start_pos.0 as u32
}
fn get_filename(tcx: TyCtxt, span: Span) -> String {
    let filename = tcx.sess.source_map().span_to_filename(span);
    if let FileName::Real(pathbuf) = &filename {
        if let Some(s) = pathbuf.to_str() {
            return s.to_string();
        }
    }
    panic!("unexpected file type: {:?}", filename);
}

pub fn map_range_to_span(tcx: TyCtxt, range: &SourceCodeRange) -> Span {
    let source_map = tcx.sess.source_map();
    let filename = FileName::Real(std::path::PathBuf::from(&range.file_name));
    let source_file = source_map.get_source_file(&filename).unwrap();
    Span::with_root_ctxt(
        BytePos(range.from + source_file.start_pos.0),
        BytePos(range.to + source_file.start_pos.0),
    )
}

pub fn map_change_from_span(
    tcx: TyCtxt,
    span: Span,
    replacement: String,
) -> Change {
    let filename = get_filename(tcx, span);
    let file_offset = get_file_offset(tcx, &filename);
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
