use crate::refactoring_invocation::{AstContext, AstDiff, QueryResult};
use crate::refactorings::visitors::ast::collect_comments;
use rustc_span::Span;

/// removes all comments with this format: /*refactor-tool:<selection>:<start/end>*/
pub fn do_refactoring(context: &AstContext, _span: Span, _add_comment: bool) -> QueryResult<AstDiff> {
    
    let mut changes = vec![];
    for span in collect_comments(&context.source())? {
        changes.push(context.map_change(span, "".to_owned())?);
    }
    Ok(AstDiff(changes))
}
