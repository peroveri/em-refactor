use rustc_span::Span;
use crate::change::FileReplaceContent;
use crate::refactor_definition::RefactoringError;
use super::utils::map_change_from_span;
use super::visitors::collect_inline_macro;

pub fn do_refactoring<'tcx>(compiler: &rustc_interface::interface::Compiler,queries:  &'tcx rustc_interface::Queries<'tcx>, span: Span) -> Result<Vec<FileReplaceContent>, RefactoringError>{

    let (crate_, ..) = 
    &*queries
        .expansion()
        .unwrap()
        .peek_mut();

    if let Some((replacement, repl_span)) = collect_inline_macro(span, crate_) {
        Ok(vec![map_change_from_span(compiler.source_map(), repl_span, replacement)])
    } else {
        Err(RefactoringError::invalid_selection(span.lo().0, span.hi().0))
    }
}
