use rustc::{declare_lint_pass, declare_lint};
use rustc::lint::{EarlyContext, EarlyLintPass, LintArray, LintPass};
// use rustc_errors::{Applicability};
use syntax::ast;

declare_lint! {
    pub MY_EARLY_LINT,
    Deny,
    "An early lint"
}

declare_lint_pass!(MyEarlyLint => [MY_EARLY_LINT]);

impl EarlyLintPass for MyEarlyLint {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, item: &ast::Expr) {
        // if let ast::ExprKind::While(..) = item.node {
        // }
    }
}