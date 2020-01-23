use rustc_span::Span;
use crate::change::Change;
use crate::refactor_definition::RefactoringError;
use super::utils::map_change_from_span;
use syntax::ast::{Expr, Stmt};
use syntax::visit::{Visitor, walk_crate };

pub fn do_refactoring<'tcx>(compiler: &rustc_interface::interface::Compiler,queries:  &'tcx rustc_interface::Queries<'tcx>, span: Span) -> Result<Vec<Change>, RefactoringError>{

    let mut v = MacroCollector {
        span,
        res: vec![],
        res_span: None
    };
    let (crate_, ..) = 
    &*queries
        .expansion()
        .unwrap()
        .peek_mut();

    walk_crate(&mut v, crate_);

    if v.res.len() > 0 {
        let r = v.res.join("");
        return Ok(vec![map_change_from_span(compiler.source_map(), v.res_span.unwrap(), r)]);
    } else {

    }
    Err(RefactoringError::file_not_found(""))
}

struct MacroCollector {
    span: Span,
    res: Vec<String>,
    res_span: Option<Span>
}

impl<'ast> Visitor<'ast> for MacroCollector {
    fn visit_expr(&mut self, ex: &'ast Expr) {
        if ex.span.from_expansion() && self.span.overlaps(ex.span.source_callsite()) {
            self.res.push(syntax::print::pprust::expr_to_string(ex));
            self.res_span = ex.span.parent();
        } else {
            syntax::visit::walk_expr(self, ex);
        }
    }
    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        if stmt.span.from_expansion() && self.span.overlaps(stmt.span.source_callsite()) {
            self.res.push(syntax::print::pprust::stmt_to_string(stmt));
            self.res_span = stmt.span.parent();
        } else {
            syntax::visit::walk_stmt(self, stmt);
        }
    }
}