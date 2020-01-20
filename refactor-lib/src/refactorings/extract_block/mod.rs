use super::utils::{map_change_from_span, get_source};
use crate::change::Change;
use crate::refactor_definition::RefactoringError;
use block_collector::collect_block;
use rustc_hir::{BodyId};
use rustc::ty::TyCtxt;
use rustc_span::Span;

mod block_collector;
mod expr_use_visit;
mod push_stmt_into_block;

fn extract_block(
    tcx: TyCtxt,
    body_id: BodyId,
    span: Span,
    source: String,
) -> Result<String, RefactoringError> {
    let (decls, ids) = push_stmt_into_block::push_stmts_into_block(tcx, body_id, span);
    let decls_fmt = decls.join(", ");
    let ids_fmt = ids.join(", ");

    // Add declaration with assignment, and expression at end of block
    // for variables declared in the selection and used later
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


/// Extract block
/// 
/// ## Algorithm
/// 
/// Steps
/// Block <- The block (innermost) containing A;B;C
/// A <- Statements before B
/// B <- Statements to be extracted
/// C <- Statements after B
/// 
/// If B ends with an expression:
///    Add { around B } and return 
/// End
/// 
/// Vs <- Locals declared in B and used in C
/// 
/// 
/// for each stmt
/// how should it be moved?
/// a. identical (cut & paste)
/// b. add declaration and assign at start of block + add var in expression at end of block
pub fn do_refactoring(tcx: TyCtxt, span: Span) -> Result<Vec<Change>, RefactoringError> {
    if let Some((block, body_id)) = collect_block(tcx, span) {
        let source = tcx.sess.source_map().span_to_snippet(span).unwrap();
        if let Some(expr) = &block.expr {
            if span.contains(expr.span) {
                return Ok(vec![map_change_from_span(tcx, span, format!("{{\n{}\n}}", source))]);
            }
        }
        Ok(vec![map_change_from_span(
            tcx,
            span,
            extract_block(tcx, body_id, span, source)?,
        )])
    } else {
        Err(RefactoringError::invalid_selection_with_code(
            span.lo().0,
            span.hi().0,
            &get_source(tcx, span)
        ))
    }
}
