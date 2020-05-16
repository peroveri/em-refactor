use rustc_hir::{BodyId, Expr, ExprKind, Param, PatKind};
use rustc_middle::ty::{TyS, print::with_crate_prefix};
use rustc_span::Span;
use crate::refactoring_invocation::{AstDiff, QueryResult, RefactoringErrorInternal, TyContext};
use crate::refactorings::visitors::hir::collect_anonymous_closure;
use refactor_lib_types::{create_refactor_tool_marker, defs::CONVERT_CLOSURE_TO_FUNCTION_FN_DEF};

/// Convert anonymous closure to function
/// 
/// For now: assume params = ident : type
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
pub fn do_refactoring(tcx: &TyContext, span: Span, add_comment: bool) -> QueryResult<AstDiff> {
    let closure = collect_anonymous_closure(tcx, span)?;

    let mut changes = vec![];
    let body = tcx.0.hir().body(closure.body_id);

    let mut new_fn = FunctionDefinition {
        ident: fresh_name(),
        params: vec![],
        output: None,
        body: tcx.get_source(body.value.span),
        body_is_block: if let ExprKind::Block(..) = body.value.kind {true} else {false}
    };

    let mut i = 0;
    for param in body.params {
        let ident = get_ident(param);
        let type_s = 
            format_ty(infer_concrete(tcx, &closure.args_1[i], closure.body_id)?);
        new_fn.params.push((ident, type_s));
        i += 1;
    }

    let out = infer_concrete(tcx, &body.value, closure.body_id)?;
    if !out.is_unit() {
        new_fn.output = Some(format_ty(out));
    }

    changes.push(tcx.map_change(closure.call_fn_expr.span, format!("({{{}{}{}\n{}}})", create_comment(add_comment, false), new_fn.formatted(), create_comment(add_comment, true), new_fn.ident))?);


    Ok(AstDiff(changes))
}

struct FunctionDefinition {
    ident: String,
    params: Vec<(String, String)>,
    output: Option<String>,
    body: String,
    body_is_block: bool
}
impl FunctionDefinition {
    fn formatted(&self) -> String {
        format!("fn {}({}){} {}", self.ident, self.format_params(), self.format_output(), self.format_body())
    }
    fn format_params(&self) -> String {
        self.params.iter().map(|(id, ty)| format!("{}: {}", id, ty)).collect::<Vec<_>>().join(", ")
    }
    fn format_output(&self) -> String {
        if let Some(s) = &self.output {
            format!(" -> {}", s)
        } else {
            "".to_owned()
        }
    }
    fn format_body(&self) -> String {
        if self.body_is_block {
            self.body.to_string()
        } else {
            format!("{{\n{}\n}}", self.body)
        }
    }
}

fn get_ident(ty: &Param) -> String {
    match ty.pat.kind {
        PatKind::Binding(_, _, id, _) => format!("{}", id),
        _ => "_".to_owned()
    }
}
fn fresh_name() -> String {"foo".to_owned()}

fn format_ty(ty: &TyS) -> String {
    with_crate_prefix(||  format!("{}", ty))
}

fn infer_concrete<'v>(tcx: &'v TyContext, expr: &Expr, body_id: BodyId) -> QueryResult<&'v TyS<'v>> {

    let def_id = tcx.0.hir().body_owner_def_id(body_id);
    let typecheck_table = tcx.0.typeck_tables_of(def_id);
    if let Some(expr_type) = typecheck_table.expr_ty_adjusted_opt(expr) {
        Ok(expr_type)
    } else {
        Err(RefactoringErrorInternal::int(&format!("Failed to get type of expression: {}", tcx.get_source(expr.span))))
    } 
}

fn create_comment(add_comment: bool, is_end: bool) -> String {
    if add_comment {
        create_refactor_tool_marker(CONVERT_CLOSURE_TO_FUNCTION_FN_DEF, is_end)
    } else {
        "".to_owned()
    }
}