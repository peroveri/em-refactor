use crate::refactoring_invocation::{FileStringReplacement, RefactoringErrorInternal, SourceCodeRange};
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

pub fn map_range_to_span(source_map: &SourceMap, range: &SourceCodeRange) -> Result<Span, RefactoringErrorInternal> {
    let filename = FileName::Real(std::path::PathBuf::from(&range.file_name));
    if let Some(source_file) = source_map.get_source_file(&filename) {
        Ok(Span::with_root_ctxt(
            BytePos(range.from + source_file.start_pos.0),
            BytePos(range.to + source_file.start_pos.0),
        ))
    } else {
        Err(RefactoringErrorInternal::file_not_found(&range.file_name))
    }
}

pub fn map_change_from_span(source_map: &SourceMap, span: Span, replacement: String) -> FileStringReplacement {
    let filename = get_filename(source_map, span);
    let file_offset = get_file_offset(source_map, &filename);
    let lines = source_map.span_to_lines(span).unwrap().lines;
    let line_start = lines.first().unwrap();
    let line_end = lines.last().unwrap();
    FileStringReplacement {
        file_name: filename,
        byte_start: span.lo().0 - file_offset,
        byte_end: span.hi().0 - file_offset,
        char_end: line_end.end_col.0,
        char_start: line_start.start_col.0,
        line_end: line_end.line_index,
        line_start: line_start.line_index,
        replacement,
    }
}

pub fn get_source(tcx: TyCtxt, span: Span) -> String {
    tcx.sess.source_map().span_to_snippet(span).unwrap()
}

pub fn get_source_from_compiler(compiler: &rustc_interface::interface::Compiler, span: Span) -> String {
    compiler.source_map().span_to_snippet(span).unwrap()
}
pub fn get_struct_hir_id(tcx: TyCtxt<'_>, field: &StructField) -> HirId {
    let struct_def_id = field.hir_id.owner_def_id();
    tcx.hir().as_local_hir_id(struct_def_id).unwrap()
}

pub fn map_span_to_index(source_map: &SourceMap, span: Span) -> (String, Range) {
    let filename = get_filename(source_map, span);
    let file_offset = get_file_offset(source_map, &filename);
    let lines = source_map.span_to_lines(span).unwrap().lines;
    let line_start = lines.first().unwrap();
    let line_end = lines.last().unwrap();
    (filename, Range {
        from: Position {
            byte: span.lo().0 - file_offset,
            character: line_start.start_col.0,
            line: line_start.line_index
        },
        to: Position {
            byte: span.hi().0 - file_offset,
            character: line_end.end_col.0,
            line: line_end.line_index
        }
    })
}

pub struct Position {
    pub byte: u32,
    pub character: usize,
    pub line: usize
}
pub struct Range {
    pub from: Position,
    pub to: Position
}