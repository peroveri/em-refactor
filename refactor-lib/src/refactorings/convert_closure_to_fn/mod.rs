// use rustc_hir::{BodyId, Expr, Pat, PatKind};
use rustc_span::Span;
use crate::refactoring_invocation::{AstDiff, QueryResult, TyContext};
// use crate::refactorings::visitors::hir::{collect_anonymous_closure};

// fn fresh_name() -> String {"foo".to_owned()}

/// Convert anonymous closure to function
/// 
/// ## Algorithm
/// 1. For each parameter
///     If the type contains infer (or "similar")
///         Run inference on the matching argument
///         Add type annotation to parameter
/// 2. If return value && return type contains infer
///     Run inference on where?
/// 
/// Change Closure Expr to block return fn
pub fn do_refactoring(_tcx: &TyContext, _span: Span) -> QueryResult<AstDiff> {
    // let closure = collect_anonymous_closure(tcx, span)?;

    let changes = vec![];

    // let body = tcx.0.hir().body(closure.body_id);
    // let mut i = 0;
    // for x in body.params {
    //     let type_s = 
    //     if contains_infer(x.pat) {
    //         infer_concrete(&closure.args_1[i], closure.body_id)
    //     } else {
    //         format!("{:?}", x.pat)
    //     };
    //     let ident = x.
    //     i += 1;
    // }


    Ok(AstDiff(changes))
}

// fn contains_infer(pat: &Pat) -> bool {
//     let mut f = false;
//     pat.walk(|p| match p.kind {
//         PatKind::Wild => {
//             f = true;
//             false
//         },
//         _ => true
//     });
//     f
// }

// fn infer_concrete(expr: &Expr, body_id: BodyId) -> String {
//     "Foo".to_owned()
// }