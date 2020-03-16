use crate::refactorings::visitors::collect_ast_block;
use rustc_ast::ast::Crate;
use rustc_span::Span;

pub fn collect_item_declarations<'v>(crate_: &Crate, span: Span) -> Option<Vec<Span>> {
    let block = collect_ast_block(crate_, span)?;
    Some(block.stmts.iter()
        .filter(|s| s.is_item())
        .map(|s| s.span)
        .collect::<Vec<_>>())
}
