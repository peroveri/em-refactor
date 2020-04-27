use super::expr_use_visit::collect_variables_declared_in_span_and_used_later;
use rustc_hir::BodyId;
use rustc_span::Span;
use crate::refactoring_invocation::{QueryResult, TyContext};

pub fn collect_variables_overlapping_span(
    tcx: &TyContext,
    body_id: BodyId,
    span: Span,
) -> QueryResult<VariablesUsedOutsideCollection> {
    let rvs = collect_variables_declared_in_span_and_used_later(tcx, body_id, span)?.get_return_values();

    let idents = rvs
        .iter()
        .map(|rv| rv.ident.to_string())
        .collect::<Vec<_>>();
    let decls = rvs
        .iter()
        .map(|rv| format!("{}{}", if rv.is_mutated { "mut " } else { "" }, rv.ident))
        .collect::<Vec<_>>();

    Ok(VariablesUsedOutsideCollection::new(decls, idents))
}

pub struct VariablesUsedOutsideCollection {
    decls: Vec<String>,
    idents: Vec<String>
}

impl VariablesUsedOutsideCollection {
    pub fn new(decls: Vec<String>, idents: Vec<String>) -> Self {
        Self {
            decls,
            idents
        }
    }

    pub fn len(&self) -> usize {
        self.decls.len()
    }

    pub fn decls_fmt(&self) -> String {
        self.decls.join(", ")
    }
    pub fn idents_fmt(&self) -> String {
        self.idents.join(", ")
    }
}
