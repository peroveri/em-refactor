use rustc_hir::{BodyId, Expr, ExprKind, Param, PatKind, Ty, TyKind};
use rustc_middle::ty::TyS;
use rustc_span::Span;
use crate::refactoring_invocation::{AstDiff, QueryResult, RefactoringErrorInternal, TyContext};
use crate::refactorings::visitors::hir::collect_anonymous_closure;

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
pub fn do_refactoring(tcx: &TyContext, span: Span, _add_comment: bool) -> QueryResult<AstDiff> {
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
        let input = &closure.fn_decl.inputs[i];
        let type_s = 
        if contains_infer(input) {
            format_ty(infer_concrete(tcx, &closure.args_1[i], closure.body_id)?)
        } else {
            rustc_hir_pretty::to_string(rustc_hir_pretty::NO_ANN, |s| s.print_type(input))
        };
        new_fn.params.push((ident, type_s));
        i += 1;
    }

    let out = infer_concrete(tcx, &body.value, closure.body_id)?;
    if !out.is_unit() {
        new_fn.output = Some(format_ty(out));
    }

    changes.push(tcx.map_change(closure.call_fn_expr.span, format!("({{{}\n{}}})", new_fn.formatted(), new_fn.ident))?);


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

fn contains_infer(pat: &Ty) -> bool {
    match pat.kind {
        TyKind::Infer => true,
        _ => false
    }
}

fn format_ty(ty: &TyS) -> String {
    format!("{}", ty)
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