use em_refactor_lib_types::FileStringReplacement;
use crate::refactoring_invocation::{QueryResult, RefactoringErrorInternal};
use rustc_middle::ty::TyCtxt;
use rustc_span::{FileName, Span};
use rustc_span::source_map::SourceMap;
use std::path::PathBuf;

pub(crate) fn get_file_offset(source_map: &SourceMap, file_name: &str) -> QueryResult<u32> {
    let file_name_real = FileName::Real(PathBuf::from(file_name.to_string()));
    let source_file = source_map.get_source_file(&file_name_real).ok_or_else(|| RefactoringErrorInternal::int(&format!("Couldn't find file: {}", file_name)))?;
    Ok(source_file.start_pos.0 as u32)
}
fn get_local_filename(source_map: &SourceMap, span: Span) -> QueryResult<String> {
    let filename = source_map.span_to_filename(span);
    let sourcefile = source_map.get_source_file(&filename).unwrap();
    if sourcefile.is_imported() {
        return Err(RefactoringErrorInternal::int(&format!("File: {:?} is not a local file", filename)));
    }
    if let FileName::Real(pathbuf) = &filename {
        if let Some(s) = pathbuf.to_str() {
            return Ok(s.to_string());
        }
    }
    Err(RefactoringErrorInternal::int(&format!("unexpected file type: {:?}", filename)))
}

fn get_filename(source_map: &SourceMap, span: Span) -> QueryResult<String> {
    let filename = source_map.span_to_filename(span);
    if let FileName::Real(pathbuf) = &filename {
        if let Some(s) = pathbuf.to_str() {
            return Ok(s.to_string());
        }
    }
    Err(RefactoringErrorInternal::int(&format!("unexpected file type: {:?}", filename)))
}

pub(crate) fn map_change_from_span(source_map: &SourceMap, span: Span, replacement: String) -> QueryResult<FileStringReplacement> {
    let filename = get_local_filename(source_map, span)?;
    let file_offset = get_file_offset(source_map, &filename)?;
    let lines = source_map.span_to_lines(span).unwrap().lines;
    let line_start = lines.first().ok_or_else(|| RefactoringErrorInternal::int(&format!("Span: {:?} had no lines", span)))?;
    let line_end = lines.last().unwrap();
    Ok(FileStringReplacement {
        file_name: filename,
        byte_start: span.lo().0 - file_offset,
        byte_end: span.hi().0 - file_offset,
        char_end: line_end.end_col.0,
        char_start: line_start.start_col.0,
        line_end: line_end.line_index,
        line_start: line_start.line_index,
        replacement,
    })
}

pub(crate) fn get_source(tcx: TyCtxt, span: Span) -> String {
    tcx.sess.source_map().span_to_snippet(span).unwrap()
}


pub(crate) fn map_span_to_index(source_map: &SourceMap, span: Span) -> QueryResult<(String, Range)> {
    let filename = get_filename(source_map, span)?;
    let file_offset = get_file_offset(source_map, &filename)?;
    let lines = source_map.span_to_lines(span).unwrap().lines;
    let line_start = lines.first().unwrap();
    let line_end = lines.last().unwrap();
    Ok((filename, Range {
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
    }))
}

pub(crate) struct Position {
    pub byte: u32,
    pub character: usize,
    pub line: usize
}
pub(crate) struct Range {
    pub from: Position,
    pub to: Position
}