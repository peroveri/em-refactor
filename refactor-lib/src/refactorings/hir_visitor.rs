use rustc::hir::{self, intravisit};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

pub struct StmtsVisitor<'v> {
    pub tcx: TyCtxt<'v>,
    pub span: Option<Span>,
    pub pos: (u32, u32)
}

// note: could be done on the AST? we only need the span of some statements
impl<'v> intravisit::Visitor<'v> for StmtsVisitor<'v> {
    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'v> {
        intravisit::NestedVisitorMap::All(&self.tcx.hir())
    }
    // visit expr, etc.

    fn visit_block(&mut self, body: &'v hir::Block) {
        if body.span.lo().0 > self.pos.0 &&
            body.span.hi().0 < self.pos.1 {
            return;
        }
        
        let from = body.stmts.iter().filter(|s| s.span.lo().0 >= self.pos.0).last();
        let to = body.stmts.iter().filter(|s| s.span.hi().0 <= self.pos.1).last();
        if let Some(from) = from {
            if let Some(to) = to {
                self.span = Some(from.span.to(to.span));
                // eprintln!("{:?}, {:?}, {:?}", span, from.span, to.span);

            }
        }
        // for stmt in &body.stmts {
        //     eprintln!("visited {:?}, {}, {}", stmt.span, stmt.span.lo().0, stmt.span.hi().0);
        // }
    }
}
