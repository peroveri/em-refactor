use super::utils::{map_change_from_span, map_range_to_span};
use crate::change::Change;
use crate::refactor_definition::SourceCodeRange;
use field_collector::collect_field;
use rustc::hir;
use rustc::ty::TyCtxt;

mod field_collector;

// translate span to (struct, field)
// then, find all bindings to the field and update
pub fn do_refactoring(tcx: TyCtxt, range: &SourceCodeRange) -> Result<Vec<Change>, String> {
    let span = map_range_to_span(tcx, range);

    if let Some(field) = collect_field(tcx, span) {
        let type_as_str = hir::print::to_string(hir::print::NO_ANN, |s| s.print_type(&*field.ty));
        let replacement = format!("Box<{}>", type_as_str);

        Ok(vec![map_change_from_span(
            tcx,
            field.ty.span,
            &range.file_name,
            replacement,
        )])
    } else {
        Err(format!( // do this on a higher level?
            "{}:{} is not a valid selection!",
            range.from, range.to
        ))
    }
}
