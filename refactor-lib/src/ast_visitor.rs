 use syntax::visit;
 use syntax::ast;

pub struct AstVisitor<'v> {
    pub krate: &'v ast::Crate
}

impl<'v> visit::Visitor<'v> for AstVisitor<'v> {
    fn visit_expr<'this>(&'this mut self, expr: &'v ast::Expr) {
    }
}