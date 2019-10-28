// Either "add decl" or "just cut & paste"
use super::expr_use_visit::{collect_vars};
use rustc::hir::BodyId;
use rustc::ty::TyCtxt;
use syntax::source_map::Span;

pub fn push_stmts_into_block(
    tcx: TyCtxt,
    body_id: BodyId,
    span: Span,
) -> Result<(Vec<String>, Vec<String>), String> {
    let rvs = collect_vars(tcx, body_id, span).get_return_values();

    let idents = rvs
        .iter()
        .map(|rv| rv.ident.to_string())
        .collect::<Vec<_>>();
    let decls = rvs
        .iter()
        .map(|rv| format!("{}{}", if rv.is_mutated { "mut " } else { "" }, rv.ident))
        .collect::<Vec<_>>();

    Ok((decls, idents))
}
