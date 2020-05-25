use rustc_span::source_map::SourceMap;
use rustc_span::{BytePos, FileName, Span};
use crate::refactoring_invocation::{QueryResult, RefactoringErrorInternal};
use em_refactor_lib_types::SelectionType;

pub struct SourceMapContext<'a> {
    pub source_map: &'a SourceMap,
}

impl<'a> SourceMapContext<'a> {
    pub(crate) fn map_selection_to_span(&self, r: SelectionType, file: String) -> QueryResult<Span> {
        match r {
            SelectionType::Comment(range_id) => {
                crate::refactorings::visitors::ast::collect_comments_with_id(self, &range_id)
            },
            SelectionType::Range(r) => {
                let tup = Self::get_int(&r)?;
                self.map_span(&file, tup.0, tup.1)
            }
        }
    }

    pub(crate) fn map_span(&self, file_name: &str, from: u32, to: u32) -> QueryResult<Span> {
        let file_name_real = FileName::Real(std::path::PathBuf::from(file_name));
        if let Some(source_file) = self.source_map.get_source_file(&file_name_real) {
            Ok(Span::with_root_ctxt(
                BytePos(from + source_file.start_pos.0),
                BytePos(to + source_file.start_pos.0),
            ))
        } else {
            Err(RefactoringErrorInternal::file_not_found(file_name))
        }
    }
    
    fn get_int(selection: &str) -> QueryResult<(u32, u32)> {
        let mut split = selection.split(':');
        if let (Some(from), Some(to)) = (split.nth(0), split.nth(0)) {
            let from = from.parse().map_err(|_| RefactoringErrorInternal::arg_def(&format!("{} is not a valid int", from)))?;
            let to = to.parse().map_err(|_| RefactoringErrorInternal::arg_def(&format!("{} is not a valid int", from)))?;
            return Ok((from, to));
        }
        Err(RefactoringErrorInternal::arg_def("Selection should be formatted as <byte_from>:<byte_to>"))
    }
    pub(crate) fn span_err(&self, span: Span, is_error: bool) -> RefactoringErrorInternal {
        RefactoringErrorInternal::invalid_selection_with_code(span.lo().0, span.hi().0, &self.get_source(span), is_error)
    }
    pub(crate) fn get_source(&self, span: Span) -> String {
        self.source_map.span_to_snippet(span).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn refactor_def_from_args() {
        let expected = Ok((1, 2));
        let actual = SourceMapContext::get_int("1:2");

        assert_eq!(actual, expected);
    }
}
