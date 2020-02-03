use syntax::ast::{Crate, Expr, Stmt};
use syntax::visit::{Visitor, walk_crate };
use rustc_span::Span;
use rustc_ast_pretty::pprust::{expr_to_string, stmt_to_string};


pub fn collect_inline_macro(span: Span, crate_: &Crate) -> Option<(String, Span)> {
    let mut v = MacroCollector {
        span,
        res: vec![],
        res_span: None
    };

    walk_crate(&mut v, crate_);

    if v.res.len() > 0 {
        Some((v.res.join(""), v.res_span.unwrap()))
    } else {
        None
    }
}

struct MacroCollector {
    span: Span,
    res: Vec<String>,
    res_span: Option<Span>
}

impl<'ast> Visitor<'ast> for MacroCollector {
    fn visit_expr(&mut self, ex: &'ast Expr) {
        if ex.span.from_expansion() && self.span.overlaps(ex.span.source_callsite()) {
            self.res.push(expr_to_string(ex));
            self.res_span = ex.span.parent();
        } else {
            syntax::visit::walk_expr(self, ex);
        }
    }
    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        if stmt.span.from_expansion() && self.span.overlaps(stmt.span.source_callsite()) {
            self.res.push(stmt_to_string(stmt));
            self.res_span = stmt.span.parent();
        } else {
            syntax::visit::walk_stmt(self, stmt);
        }
    }
}