use rustc::{declare_lint_pass, declare_lint};
use rustc::lint::{LateContext, LateLintPass, LintArray, LintPass};
// use rustc_errors::{Applicability};
use rustc::hir;

declare_lint! {
    pub MY_LATE_LINT,
    Warn,
    "A late lint"
}

declare_lint_pass!(MyLateLint => [MY_LATE_LINT]);

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for MyLateLint {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, item: &'tcx hir::Expr) {
        // if let hir::ExprKind::Loop(.., hir::LoopSource::WhileLet) = item.node {
            // let snippet = cx.sess().source_map().span_to_snippet(block.span).ok().unwrap();
            // let mut db = cx.struct_span_lint(MY_LATE_LINT, block_span, msg);
            // db.span_suggestion(block_span, "message", snippet, Applicability::MachineApplicable);
            // db.emit();
        // }
    }
}