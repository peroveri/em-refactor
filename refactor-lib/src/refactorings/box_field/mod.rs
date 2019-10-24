use super::utils::{map_change_from_span, map_range_to_span};
use crate::change::Change;
use crate::refactor_definition::SourceCodeRange;
use field_collector::collect_field;
use rustc::hir;
use rustc::ty::TyCtxt;
use syntax_pos::Span;

use super::utils::get_source;

// mod binds_to_field_collector;
mod field_collector;
mod function_body_collector;

fn get_field_change(tcx: &TyCtxt, range: &SourceCodeRange, field: &hir::StructField) -> Change {
    let type_as_str = hir::print::to_string(hir::print::NO_ANN, |s| s.print_type(&*field.ty));
    let replacement = format!("Box<{}>", type_as_str);

    map_change_from_span(tcx, field.ty.span, &range.file_name, replacement)
}

fn get_read_change(tcx: &TyCtxt, span: Span) -> String {
    let original = get_source(&tcx, span);
    format!("*{}", original)
}
fn get_write_change(tcx: TyCtxt, span: Span) -> String {
    let original = get_source(&tcx, span);
    format!("Box::new({})", original)
}

fn get_use_changes<'tcx>(
    tcx: TyCtxt<'tcx>,
    file_name: &str,
    field: &hir::StructField,
) -> Vec<Change> {
    let bodies = function_body_collector::collect_function_bodies(tcx);
    // let (reads, writes) = binds_to_field_collector::run_on_all_bodies(tcx, &bodies, field.span);
    let mut r = vec![];
    // r.extend(reads.iter().map(|read| map_change_from_span(&tcx, *read, file_name, get_read_change(tcx, *read))));
    // r.extend(writes.iter().map(|write| map_change_from_span(&tcx, *write, file_name, get_write_change(tcx, *write))));
    eprintln!("{:?}", r);
    r
}

// translate span to (struct, field)
// then, find all bindings to the field and update
pub fn do_refactoring(tcx: TyCtxt, range: &SourceCodeRange) -> Result<Vec<Change>, String> {
    let span = map_range_to_span(tcx, range);

    if let Some(field) = collect_field(tcx, span) {
        let mut changes = get_use_changes(tcx, &range.file_name, field);
        changes.push(get_field_change(&tcx, range, field));
        Ok(changes)
    } else {
        Err(format!( // do this on a higher level?
            "{}:{} is not a valid selection!",
            range.from, range.to
        ))
    }
}
