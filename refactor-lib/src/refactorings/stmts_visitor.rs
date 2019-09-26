use rustc::hir::{self, intravisit};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

pub struct ExtractMethodStatements<'v> {
    pub mod_: &'v hir::Mod,
    pub fn_body_id: hir::BodyId,
    pub S0: Vec<&'v hir::Stmt>,
    pub Si: Vec<&'v hir::Stmt>,
    pub Sj: Vec<&'v hir::Stmt>
}

/**
 * Given a selection (byte start, byte end) and file name, this visitor finds 
 * the sequence of statements starting and ending at selection start and end.
 */
pub struct StmtsVisitor<'v> {
    pub tcx: TyCtxt<'v>,
    pub file: String,
    pub pos: (u32, u32),
    pub stmts: Option<ExtractMethodStatements<'v>>,
    mod_: Option<&'v hir::Mod>,
    fn_body_id: Option<hir::BodyId>
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
            mod_: None,
            fn_body_id: None
        };

        intravisit::walk_crate(&mut v, tcx.hir().krate());
        v
    }
}

/**
 * byte 0 ...      byte i
 * <stmt start>xxx;<statment end> 
 * 
 * byte j
 * 
 * Note: need to handle blocks within statements etc.
 * 
 */

impl<'v> intravisit::Visitor<'v> for StmtsVisitor<'v> {
    fn nested_visit_map<'this>(&'this mut self) -> intravisit::NestedVisitorMap<'this, 'v> {
        intravisit::NestedVisitorMap::All(&self.tcx.hir())
    }

    fn visit_mod(&mut self, mod_: &'v hir::Mod, _span: Span, hir_id: hir::HirId) {
        self.mod_ = Some(mod_);
        intravisit::walk_mod(self, mod_, hir_id);
    }

    fn visit_fn(&mut self, fk: hir::intravisit::FnKind<'v>, fd: &'v hir::FnDecl, b: hir::BodyId, s: Span, id: hir::HirId) {
        self.fn_body_id = Some(b);
        intravisit::walk_fn(self, fk, fd, b, s, id);
    }

    fn visit_block(&mut self, body: &'v hir::Block) {
        let (from, to) = self.pos;
        if !self.is_same_file(body.span) {
            intravisit::walk_block(self, body);
            return;
        }
        if body.span.lo().0 > from ||
            body.span.hi().0 < to {
            intravisit::walk_block(self, body);
            return;
        }

        let stmts = &body.stmts;
        let start_index = stmts.iter().position(|s| s.span.lo().0 >= from).unwrap();
        let end_index = stmts.iter().rposition(|s| s.span.hi().0 <= to).unwrap();
        // let mut iter = body.stmts.iter();

        self.stmts = Some(ExtractMethodStatements {
            S0: stmts.iter().take(start_index).collect(), //.take_while(|s| s.span.lo().0 < from).collect(),
            Si: stmts.iter().skip(start_index).take(1 + end_index - start_index).collect(),
            Sj: stmts.iter().skip(end_index + 1).collect(),
            mod_: self.mod_.unwrap(),
            fn_body_id: self.fn_body_id.unwrap()
        });

    }
}
