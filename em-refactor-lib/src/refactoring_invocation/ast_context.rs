use rustc_ast::ast::Crate;
use rustc_interface::interface::Compiler;
use rustc_interface::Queries;
use rustc_span::Span;
use rustc_span::source_map::SourceMap;
use crate::refactorings::utils::map_change_from_span;
use em_refactor_lib_types::FileStringReplacement;
use super::{QueryResult, SourceMapContext};

pub struct AstContext<'a, 'b> {
    pub compiler: &'a Compiler,
    pub queries: &'b Queries<'b>,
    pub crate_: Option<Crate>
}

impl<'a, 'b> AstContext<'a, 'b> {
    pub(crate) fn source(&self) -> SourceMapContext<'a> {
        SourceMapContext {
            source_map: self.compiler.source_map()
        }
    }
    pub(crate) fn get_source_map(&self) -> &SourceMap {
        self.compiler.session().source_map()
    }
    pub(crate) fn get_source(&self, span: Span) -> String {
        self.source().get_source(span)
    }

    pub(crate) fn map_change(&self, span: Span, replacement: String) -> QueryResult<FileStringReplacement> {
        map_change_from_span(self.compiler.source_map(), span, replacement)
    }
    pub(crate) fn load_crate(&mut self) {
        let crate_ = self.queries
        .expansion()
        .unwrap()
        .peek().0.clone();
        self.crate_ = Some(crate_);
    }
    pub(crate) fn get_crate(&self) -> &Crate {
        match &self.crate_ {
            Some(c) => c,
            _ => panic!()
        }
    }
    pub(crate) fn new(compiler: &'a Compiler, queries: &'b Queries<'b>) -> Self {
        Self {
            compiler,
            queries,
            crate_: None
        }
    }
}