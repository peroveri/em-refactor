use rustc_hir::{Arm, Block, BodyId, ExprKind, FnDecl, HirId, MatchSource, Mod, Stmt};
use rustc_hir::intravisit::{walk_block, walk_crate, Visitor, NestedVisitorMap, walk_mod, FnKind, walk_fn, walk_expr};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;

pub struct ExtractMethodStatements<'v> {
    pub mod_: &'v Mod<'v>,
    pub fn_body_id: BodyId,
    pub stmts: Vec<&'v Stmt<'v>>,
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
    mod_: Option<&'v Mod<'v>>,
    fn_body_id: Option<BodyId>,
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

    walk_crate(&mut v, tcx.hir().krate());
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

impl<'v> Visitor<'v> for StmtsVisitor<'v> {
    type Map = Map<'v>;
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, Self::Map> {
        NestedVisitorMap::All(&self.tcx.hir())
    }

    fn visit_mod(&mut self, mod_: &'v Mod, _span: Span, hir_id: HirId) {
        self.mod_ = Some(mod_);
        walk_mod(self, mod_, hir_id);
    }

    fn visit_fn(
        &mut self,
        fk: FnKind<'v>,
        fd: &'v FnDecl,
        b: BodyId,
        s: Span,
        id: HirId,
    ) {
        self.fn_body_id = Some(b);
        self.fn_decl_pos = s.lo().0;
        walk_fn(self, fk, fd, b, s, id);
    }

    fn visit_block(&mut self, body: &'v Block) {
        if let Some(expr) = &body.expr {
            if let ExprKind::Match(_, ref arms, MatchSource::WhileDesugar) = (*expr).kind
            {
                if let Some(arm) = arms.first() {
                    let Arm { body, .. } = arm;
                    walk_expr(self, &**body);
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
            walk_block(self, body);
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
