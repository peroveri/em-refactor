use rustc_ast::ast::Crate;
use rustc_interface::interface::Compiler;
use rustc_interface::Queries;
use rustc_span::Span;
use rustc_span::source_map::SourceMap;
use crate::refactorings::utils::{map_range_to_span, map_change_from_span};
use crate::output_types::FileStringReplacement;
use super::{RefactoringErrorInternal, SourceCodeRange};

pub struct AstContext<'a, 'b> {
    pub compiler: &'a Compiler,
    pub queries: &'b Queries<'b>,
    pub crate_: Option<Crate>
}

impl<'a, 'b> AstContext<'a, 'b> {
    pub fn map_range_to_span(&self, range: &SourceCodeRange) -> Result<Span, RefactoringErrorInternal> {
        map_range_to_span(self.compiler.session().source_map(), range)
    }
    pub fn get_source_map(&self) -> &SourceMap {
        self.compiler.session().source_map()
    }
    pub fn get_source(&self, span: Span) -> String {
        self.compiler.source_map().span_to_snippet(span).unwrap()
    }

    pub fn map_change(&self, span: Span, replacement: String) -> FileStringReplacement {
        map_change_from_span(self.compiler.source_map(), span, replacement)
    }
    pub fn load_crate(&mut self) {
        let crate_ = self.queries
        .expansion()
        .unwrap()
        .peek().0.clone();
        self.crate_ = Some(crate_);
    }
    pub fn get_crate(&self) -> &Crate {
        match &self.crate_ {
            Some(c) => c,
            _ => panic!()
        }
    }
    pub fn new(compiler: &'a Compiler, queries: &'b Queries<'b>) -> Self {
        Self {
            compiler,
            queries,
            crate_: None
        }
    }
    pub fn span_err(&self, span: Span) -> RefactoringErrorInternal {
        RefactoringErrorInternal::invalid_selection_with_code(span.lo().0, span.hi().0, &self.get_source(span))
    }
}