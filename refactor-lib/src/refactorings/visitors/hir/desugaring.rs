use rustc_hir::{Arm, ExprKind, MatchSource};
use rustc_hir::intravisit::{Visitor, walk_expr};

pub fn walk_desugars<'v, T>(visitor: &mut T, kind: &ExprKind<'v>) where T: Visitor<'v> {
    if let ExprKind::Match(_, ref arms, MatchSource::WhileDesugar) = kind
    {
        if let Some(arm) = arms.first() {
            let Arm { body, .. } = arm;
            walk_expr(visitor, &**body);
        }
    }
    if let ExprKind::Match(_, ref arms, MatchSource::WhileLetDesugar) = kind
    {
        if let Some(arm) = arms.first() {
            let Arm { body, .. } = arm;
            walk_expr(visitor, &**body);
        }
    }
}
