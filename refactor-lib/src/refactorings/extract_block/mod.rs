use crate::change::Change;
use crate::refactor_definition::SourceCodeRange;
use crate::refactorings::extract_block::block_collector::collect_block;
use crate::refactorings::utils::{get_file_offset, map_range_to_span};
use rustc::hir;
use rustc::ty::{self, TyCtxt};
use syntax_pos::Span;

mod block_collector;
mod push_stmt_into_block;

fn extract_block(
    tcx: TyCtxt,
    body_id: hir::BodyId,
    span: Span,
    source: String,
) -> Result<String, String> {
    let (decls, ids) = push_stmt_into_block::push_stmts_into_block(tcx, body_id, span)?;
    let decls_fmt = decls.join(", ");
    let ids_fmt = ids.join(", ");

    let (let_b, expr, end) = match ids.len() {
        0 => ("".to_owned(), "".to_owned(), "".to_owned()),
        1 => (format!("let {} = \n", decls_fmt), ids_fmt, ";".to_owned()),
        _ => (
            format!("let ({}) = \n", decls_fmt),
            format!("({})", ids_fmt),
            ";".to_owned(),
        ),
    };
    Ok(format!("{}{{\n{}\n{}}}{}", let_b, source, expr, end))
}

/// for each stmt
/// how should it be moved?
/// a. identical (cut & paste)
/// b. add declaration and assign at start of block + add var in expression at end of block
pub fn do_refactoring(tcx: ty::TyCtxt, range: &SourceCodeRange) -> Result<Vec<Change>, String> {
    let span = map_range_to_span(tcx, range);
    let file_offset = get_file_offset(tcx, &range.file_name);
    let block = collect_block(tcx, span);

    if let Some((block, body_id)) = block {
        let source = tcx.sess.source_map().span_to_snippet(span).unwrap();
        if let Some(expr) = &block.expr {
            if span.contains(expr.span) {
                return Ok(vec![Change {
                    file_name: range.file_name.to_string(),
                    file_start_pos: file_offset,
                    start: range.from,
                    end: range.to,
                    replacement: format!("{{\n{}\n}}", source),
                }]);
            }
        }
        Ok(vec![Change {
            file_name: range.file_name.to_string(),
            file_start_pos: file_offset,
            start: range.from,
            end: range.to,
            replacement: extract_block(tcx, body_id, span, source)?,
        }])
    } else {
        Err(format!( // do this on a higher level?
            "{}:{} is not a valid selection!",
            range.from, range.to
        ))
    }
}
