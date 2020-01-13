// Either "add decl" or "just cut & paste"
use super::expr_use_visit::{collect_vars};
use rustc_hir::BodyId;
use rustc::ty::TyCtxt;
use rustc_span::Span;

pub fn push_stmts_into_block(
    tcx: TyCtxt,
    body_id: BodyId,
    span: Span,
) -> (Vec<String>, Vec<String>) {
    let rvs = collect_vars(tcx, body_id, span).get_return_values();

    let idents = rvs
        .iter()
        .map(|rv| rv.ident.to_string())
        .collect::<Vec<_>>();
    let decls = rvs
        .iter()
        .map(|rv| format!("{}{}", if rv.is_mutated { "mut " } else { "" }, rv.ident))
        .collect::<Vec<_>>();

    (decls, idents)
}
