use rustc_hir::{Arm, Block, Expr, ExprKind, LoopSource, MatchSource, Pat, StmtKind};
use rustc_hir::intravisit::{Visitor, walk_expr};
use if_chain::if_chain;

pub fn walk_desugars<'v, T>(visitor: &mut T, expr: &'v Expr) -> bool where T: Visitor<'v> {

    if let Some((_pat, arg, body)) = for_loop(expr) {
        walk_expr(visitor, arg);
        walk_expr(visitor, body);
        true
    // } else if let Some((cond, body)) = while_let(expr) {
    //     walk_expr(visitor, cond);
    //     walk_expr(visitor, body);
    //     true
    } else if let Some((cond, then, els)) = if_block(expr) {
        walk_expr(visitor, cond);
        walk_expr(visitor, then);
        if let Some(e) = els {
            walk_expr(visitor, e);
        }
        true
    } else if let Some((cond, body)) = while_loop(expr) {
        walk_expr(visitor, cond);
        walk_expr(visitor, body);
        true
    } else {
        false
    }
}

/// from clippy
/// Recover the essential nodes of a desugared for loop:
/// `for pat in arg { body }` becomes `(pat, arg, body)`.
fn for_loop<'tcx>(
    expr: &'tcx Expr<'tcx>,
) -> Option<(&Pat<'_>, &'tcx Expr<'tcx>, &'tcx Expr<'tcx>)> {
    if_chain! {
        if let ExprKind::Match(ref iterexpr, ref arms, MatchSource::ForLoopDesugar) = expr.kind;
        if let ExprKind::Call(_, ref iterargs) = iterexpr.kind;
        if iterargs.len() == 1 && arms.len() == 1 && arms[0].guard.is_none();
        if let ExprKind::Loop(ref block, _, _) = arms[0].body.kind;
        if block.expr.is_none();
        if let [ _, _, ref let_stmt, ref body ] = *block.stmts;
        if let StmtKind::Local(ref local) = let_stmt.kind;
        if let StmtKind::Expr(ref expr) = body.kind;
        then {
            return Some((&*local.pat, &iterargs[0], expr));
        }
    }
    None
}
/// from clippy
/// Recover the essential nodes of a desugared while loop:
/// `while cond { body }` becomes `(cond, body)`.
fn while_loop<'tcx>(expr: &'tcx Expr<'tcx>) -> Option<(&'tcx Expr<'tcx>, &'tcx Expr<'tcx>)> {
    if_chain! {
        if let ExprKind::Loop(block, _, LoopSource::While) = &expr.kind;
        if let Block { expr: Some(expr), .. } = &**block;
        if let ExprKind::Match(cond, arms, MatchSource::WhileDesugar) = &expr.kind;
        if let ExprKind::DropTemps(cond) = &cond.kind;
        if let [arm, ..] = &arms[..];
        if let Arm { body, .. } = arm;
        then {
            return Some((cond, body));
        }
    }
    None
}

/// from clippy
/// Recover the essential nodes of a desugared if block
/// `if cond { then } else { els }` becomes `(cond, then, Some(els))`
fn if_block<'tcx>(
    expr: &'tcx Expr<'tcx>,
) -> Option<(
    &'tcx Expr<'tcx>,
    &'tcx Expr<'tcx>,
    Option<&'tcx Expr<'tcx>>,
)> {
    if let ExprKind::Match(ref cond, ref arms, MatchSource::IfDesugar { contains_else_clause }) = expr.kind {
        let cond = if let ExprKind::DropTemps(ref cond) = cond.kind {
            cond
        } else {
            panic!("If block desugar must contain DropTemps");
        };
        let then = &arms[0].body;
        let els = if contains_else_clause {
            Some(&*arms[1].body)
        } else {
            None
        };
        Some((cond, then, els))
    } else {
        None
    }
}

// fn while_let<'tcx>(
//     expr: &'tcx Expr<'tcx>,
// ) -> Option<(
//     &'tcx Expr<'tcx>,
//     &'tcx Expr<'tcx>,
// )> {
//     if let ExprKind::Match(ref cond, ref arms, MatchSource::WhileLetDesugar) = expr.kind {
        
//         if let Some(arm) = arms.get(1) {
//             let Arm { body, .. } = arm;
//             return Some((cond, body));
//         }
//     } 
//     None
// }