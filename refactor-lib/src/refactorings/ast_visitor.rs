 use syntax::visit;
 use syntax::ast;
 use syntax::source_map;

pub struct AstVisitor<'v> {
    pub krate: &'v ast::Crate,
    pub source_map: &'v source_map::SourceMap
}

impl<'v> visit::Visitor<'v> for AstVisitor<'v> {
    fn visit_expr<'this>(&'this mut self, expr: &'v ast::Expr) {
        fn z () {}
        self.visit_expr(expr);
    }
    fn visit_fn<'this>(&'this mut self, fk: visit::FnKind<'this>, fd: &'this ast::FnDecl, s: source_map::Span, _: ast::NodeId) {
        eprintln!("{:?}", fd);
        eprintln!("{:?}", s);

        eprintln!("{}", self.source_map.span_to_snippet(s).unwrap());

        // self.krate.
        
        // eprintln!("{:?}", fk);
    }
}