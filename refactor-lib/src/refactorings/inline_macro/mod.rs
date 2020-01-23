use rustc_span::Span;
use crate::change::Change;
use crate::refactor_definition::RefactoringError;
use syntax::ast::{Crate, Expr, Stmt};
use syntax::visit::{Visitor, walk_crate };

pub fn do_refactoring<'tcx>(compiler: &rustc_interface::interface::Compiler,queries:  &'tcx rustc_interface::Queries<'tcx>, span: Span) -> Result<Vec<Change>, RefactoringError>{

    let mut v = MacroCollector {
        span,
        res: vec![],
        res_span: None
    };
    let (crate_, ..) = 
    &*queries
        .expansion()
        .unwrap()
        .peek_mut();

    walk_crate(&mut v, crate_);

    if v.res.len() > 0 {
        let r = v.res.join("");
        return Ok(vec![map_change_from_span(compiler.source_map(), v.res_span.unwrap(), r)]);
    } else {

    }
    Err(RefactoringError::file_not_found(""))
}

struct MacroCollector {
    span: Span,
    res: Vec<String>,
    res_span: Option<Span>
}

impl<'ast> Visitor<'ast> for MacroCollector {
    fn visit_expr(&mut self, ex: &'ast Expr) {
        if ex.span.from_expansion() && self.span.overlaps(ex.span.source_callsite()) {
            self.res.push(syntax::print::pprust::expr_to_string(ex));
            self.res_span = ex.span.parent();
        } else {
            syntax::visit::walk_expr(self, ex);
        }
    }
    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        if stmt.span.from_expansion() && self.span.overlaps(stmt.span.source_callsite()) {
            self.res.push(syntax::print::pprust::stmt_to_string(stmt));
            self.res_span = stmt.span.parent();
        } else {
            syntax::visit::walk_stmt(self, stmt);
        }
    }
}
use std::path::PathBuf;
use rustc_span::source_map::SourceMap;
use rustc_span::{BytePos, FileName};

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
fn map_change_from_span(source_map: &SourceMap, span: Span, replacement: String) -> Change {

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
