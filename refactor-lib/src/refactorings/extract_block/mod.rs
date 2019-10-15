use crate::change::Change;
use crate::refactor_definition::SourceCodeRange;
use crate::refactorings::extract_block::block_collector::collect_block;
use crate::refactorings::extract_method::map_to_span;
use rustc::ty;

mod block_collector;
mod push_stmt_into_block;

struct ExtractBlockResult {
    ids_to_decl: Vec<String>,
}

/// for each stmt
/// how should it be moved?
/// a. identical (cut & paste)
/// b. add declaration and assign at start of block + add var in expression at end of block
pub fn do_refactoring(tcx: ty::TyCtxt, range: &SourceCodeRange) -> Result<Vec<Change>, String> {
    let span = map_to_span(
        tcx.sess.source_map(),
        (range.from, range.to),
        &range.file_name,
    );
    let file_name = syntax::source_map::FileName::Real(std::path::PathBuf::from(
        range.file_name.to_string(),
    ));
    let source_file = tcx.sess.source_map().get_source_file(&file_name).unwrap();

    let block = collect_block(tcx, span);

    if let Some(block) = block {
        if let Some(ref expr) = block.expr {
            if span.contains(expr.span) {
                return Ok(vec![
                    Change {
                        file_name: range.file_name.to_string(),
                        file_start_pos: source_file.start_pos.0 as u32,
                        start: range.from,
                        end: range.to,
                        replacement: format!("{{{}}}", "")
                    }
                ])
            }
        } else {
        }

        Err("todo".to_owned())
    } else {
        Err(format!( // do this on a higher level?
            "{}:{} is not a valid selection!",
            range.from, range.to
        ))
    }
}
