use rustc::hir::{self, intravisit};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

pub struct ExtractMethodStatements<'v> {
    pub mod_: &'v hir::Mod,
    pub fn_body_id: hir::BodyId,
    pub stmts: Vec<&'v hir::Stmt>,
    pub fn_decl_pos: u32,
}

/**
 * Given a selection (byte start, byte end) and file name, this visitor finds
 * the sequence of statements starting and ending at selection start and end.
 */
struct StmtsVisitor<'v> {
    tcx: TyCtxt<'v>,
    pos: Span,
    stmts: Option<ExtractMethodStatements<'v>>,
    fn_decl_pos: u32,
    mod_: Option<&'v hir::Mod>,
    fn_body_id: Option<hir::BodyId>,
}

pub fn visit_stmts(tcx: TyCtxt, pos: Span) -> Option<ExtractMethodStatements> {
    let mut v = StmtsVisitor {
        tcx,
        pos,
        stmts: None,
        mod_: None,
        fn_decl_pos: 0,
        fn_body_id: None,
    };

    intravisit::walk_crate(&mut v, tcx.hir().krate());
    v.stmts
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

    fn visit_fn(
        &mut self,
        fk: intravisit::FnKind<'v>,
        fd: &'v hir::FnDecl,
        b: hir::BodyId,
        s: Span,
        id: hir::HirId,
    ) {
        self.fn_body_id = Some(b);
        self.fn_decl_pos = s.lo().0;
        intravisit::walk_fn(self, fk, fd, b, s, id);
    }

    fn visit_block(&mut self, body: &'v hir::Block) {
        if let Some(expr) = &body.expr {
            if let hir::ExprKind::Match(_, ref arms, hir::MatchSource::WhileDesugar) = (*expr).kind
            {
                if let Some(arm) = arms.first() {
                    let hir::Arm { body, .. } = arm;
                    intravisit::walk_expr(self, &**body);
                }
            }
        }
        if !body.span.contains(self.pos) {
            return;
        }

        let stmts = body
            .stmts
            .iter()
            .filter(|s| self.pos.contains(s.span))
            .collect::<Vec<_>>();
        if stmts.is_empty() {
            intravisit::walk_block(self, body);
            return;
        }

        self.stmts = Some(ExtractMethodStatements {
            stmts,
            mod_: self.mod_.unwrap(),
            fn_body_id: self.fn_body_id.unwrap(),
            fn_decl_pos: self.fn_decl_pos, // TODO: is possibly wrong if there is a fn decl inside the block
        });
    }
}
