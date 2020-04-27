// Either "add decl" or "just cut & paste"
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

    fn decls_fmt(&self) -> String {
        self.decls.join(", ")
    }
    fn idents_fmt(&self) -> String {
        self.idents.join(", ")
    }
    /// (let statement, expression, semicolon?)
    pub fn get_let_expr_end(&self) -> (String, String, String) {
        match self.idents.len() {
            0 => ("".to_owned(), "".to_owned(), "".to_owned()),
            1 => (format!("let {} = \n", self.decls_fmt()), self.idents_fmt(), ";".to_owned()),
            _ => (
                format!("let ({}) = \n", self.decls_fmt()),
                format!("({})", self.idents_fmt()),
                ";".to_owned(),
            ),
        }
    }
}
