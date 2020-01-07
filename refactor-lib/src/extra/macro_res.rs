use rustc_span::Span;
use syntax::ast::{Crate, Expr};
use syntax::visit::Visitor;

struct MacroCollector {
    span: Span,
    res: bool
}

impl<'ast> Visitor<'ast> for MacroCollector {
    fn visit_expr(&mut self, ex: &'ast Expr) {
        if let Some(parent) = ex.span.parent() {
            if parent.contains(self.span) {
                let ex_str = syntax::print::pprust::expr_to_string(ex);
                
                self.res = true;
                print!("{}", serde_json::json!([{
                    "type": ex_str
                }]));
                return;
            }
        }
        syntax::visit::walk_expr(self, ex);
    }
}

pub fn resolve(crate_: &Crate, span: Span) -> bool {
    let mut collector = MacroCollector {
        span,
        res: false
    };

    syntax::visit::walk_crate(&mut collector, crate_);

    collector.res
}