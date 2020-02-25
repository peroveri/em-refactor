use super::utils::{map_change_from_span, get_source};
use item_declaration_collector::collect_item_declarations;
use crate::refactoring_invocation::{FileStringReplacement, RefactoringErrorInternal};
use rustc::ty::TyCtxt;
use rustc_span::{BytePos, Span};

mod item_declaration_collector;

/// Given a selection within a block, contiguous statements (0..n) and an expression (0|1)
/// It should pull up item declarations occuring at this block level
/// These item declarations can only be found in the selection of statements (if they are item decls.)
/// 
/// (Maybe?) Only if the item declarations are used outside the selection (before or after)
pub fn do_refactoring(tcx: TyCtxt, span: Span) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {
    if let Some(item_declarations) = collect_item_declarations(tcx, span) {
        let source_map = tcx.sess.source_map();
        let mut res = vec![];

        res.push(map_change_from_span(
            source_map,
            span.with_lo(span.lo() - BytePos(0)).shrink_to_lo(),
            item_declarations.items.iter().map(|s| get_source(tcx, *s)).collect::<Vec<_>>().join("\n")
        ));
        for delete in item_declarations.items {
            res.push(map_change_from_span(
                source_map,
                delete,
                "".to_owned(),
            ));
        }
        if res.len() == 0 {
            res.push(map_change_from_span(
                source_map,
                span.shrink_to_lo(),
                "".to_owned(),
            ));
        }
        Ok(res)
    } else {
        Err(RefactoringErrorInternal::invalid_selection_with_code(
            span.lo().0,
            span.hi().0,
            &get_source(tcx, span)
        ))
    }
}
