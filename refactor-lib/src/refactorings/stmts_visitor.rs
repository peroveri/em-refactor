use rustc::hir::{self, intravisit};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

/**
 * Given a selection (byte start, byte end) and file name, this visitor finds 
 * the sequence of statements starting and ending at selection start and end.
 */
pub struct StmtsVisitor<'v> {
    pub tcx: TyCtxt<'v>,
    pub file: String,
    pub pos: (u32, u32),
    pub stmts: Option<Vec<&'v hir::Stmt>>,
    pub mod_: Option<&'v hir::Mod>
}

impl StmtsVisitor<'_> {
    fn is_same_file(&self, span: Span) -> bool {
        let file_name = self.tcx.sess.source_map().span_to_filename(span);
        if let syntax::source_map::FileName::Real(real) = file_name {
            return real.to_str() == Some(&self.file);
        }
        false
    }

    pub fn visit<'v>(tcx: TyCtxt<'v>, file: &str, pos: (u32, u32))-> StmtsVisitor<'v> {
        let mut v = StmtsVisitor {
            tcx,
            file: file.to_string(),
            pos,
            stmts: None,
            mod_: None
        };

        intravisit::walk_crate(&mut v, tcx.hir().krate());
        v
    }
}

impl<'v> intravisit::Visitor<'v> for StmtsVisitor<'v> {
    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'v> {
        intravisit::NestedVisitorMap::All(&self.tcx.hir())
    }

    fn visit_mod(&mut self, mod_: &'v hir::Mod, _span: Span, hir_id: hir::HirId) {
        self.mod_ = Some(mod_);
        intravisit::walk_mod(self, mod_, hir_id);
    }

    fn visit_block(&mut self, body: &'v hir::Block) {
        let (from, to) = self.pos;
        if !self.is_same_file(body.span) {
            intravisit::walk_block(self, body);
            return;
        }
        if body.span.lo().0 > from &&
            body.span.hi().0 < to {
            intravisit::walk_block(self, body);
            return;
        }

        let stmts = body.stmts.iter().skip_while(|s| s.span.lo().0 < from)
            .take_while(|s| s.span.hi().0 <= to)
            .collect::<Vec<_>>();

        self.stmts = Some(stmts);
    }
}
