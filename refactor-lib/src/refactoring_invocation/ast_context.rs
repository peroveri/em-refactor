use rustc_interface::interface::Compiler;
use rustc_interface::Queries;
use rustc_span::Span;
use crate::refactorings::utils::{map_range_to_span, map_change_from_span};
use super::{RefactoringErrorInternal, SourceCodeRange, FileStringReplacement};

pub struct AstContext<'a, 'b> {
    pub compiler: &'a Compiler,
    pub queries: &'b Queries<'b>
}

impl<'a, 'b> AstContext<'a, 'b> {
    pub fn map_range_to_span(&self, range: &SourceCodeRange) -> Result<Span, RefactoringErrorInternal> {
        map_range_to_span(self.compiler.session().source_map(), range)
    }
    pub fn get_source(&self, span: Span) -> String {
        self.compiler.source_map().span_to_snippet(span).unwrap()
    }

    pub fn map_change(&self, span: Span, replacement: String) -> FileStringReplacement {
        map_change_from_span(self.compiler.source_map(), span, replacement)
    }
    pub fn new(compiler: &'a Compiler, queries: &'b Queries<'b>) -> Self {
        Self {
            compiler,
            queries
        }
    }
}