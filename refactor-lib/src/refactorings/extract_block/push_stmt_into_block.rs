// Either "add decl" or "just cut & paste"
use super::expr_use_visit::{collect_vars};
use rustc_hir::BodyId;
use rustc::ty::TyCtxt;
use rustc_span::Span;

pub fn collect_variables_overlapping_span(
    tcx: TyCtxt,
    body_id: BodyId,
    span: Span,
) -> VariablesUsedOutsideCollection {
    let rvs = collect_vars(tcx, body_id, span).get_return_values();

    let idents = rvs
        .iter()
        .map(|rv| rv.ident.to_string())
        .collect::<Vec<_>>();
    let decls = rvs
        .iter()
        .map(|rv| format!("{}{}", if rv.is_mutated { "mut " } else { "" }, rv.ident))
        .collect::<Vec<_>>();

    VariablesUsedOutsideCollection::new(decls, idents)
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
