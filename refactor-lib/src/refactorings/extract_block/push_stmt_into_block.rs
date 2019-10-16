// Either "add decl" or "just cut & paste"
use crate::refactorings::extract_method::expr_use_visit::{collect_vars, CollectVarsArgs};
use rustc::hir::BodyId;
use rustc::ty::TyCtxt;
use syntax::source_map::Span;

pub fn push_stmts_into_block(
    tcx: TyCtxt,
    body_id: BodyId,
    span: Span,
) -> Result<Vec<String>, String> {
    let res = collect_vars(tcx, CollectVarsArgs { body_id, spi: span });

    Ok(res
        .get_return_values()
        .iter()
        .map(|rv| rv.ident.to_string())
        .collect::<Vec<_>>())
}
