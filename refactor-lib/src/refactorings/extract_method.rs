use rustc::hir::intravisit;
use rustc::ty;
use rustc::hir;
use syntax::source_map::SourceMap;
use crate::refactorings::hir_visitor::StmtsVisitor;
use syntax_pos::Span;

/**
 * A naive and incorrect algorithm for extract method
 * 
 * Extract method
 * 
 * Input:
 * - f - A function 
 * - m - The module containing f
 * - s - A selection in f (of consecutive statements?)
 * 
 * Assumptions:
 * - f is not a method
 * 
 * Steps:
 * g <- new function with fresh name
 * add g to m
 * vs <- all variables in s not declared in s
 * add vs as parameters of g
 * replace s with a call to g with arguments vs
 */

pub struct ExtractMethodRefactoring<'v> {
    pub tcx: ty::TyCtxt<'v>,
}

impl<'v> ExtractMethodRefactoring<'v> {

    pub fn extract_method(&self, file_name: &str, pos: (u32, u32)) -> Option<Span> {
        let mut stmt_visit = StmtsVisitor {
            tcx: self.tcx,
            span: None,
            pos
        };

        intravisit::walk_crate(&mut stmt_visit, self.tcx.hir().krate());

        return stmt_visit.span;
    }
}

impl<'v> ExtractMethodRefactoring<'v> {



    pub fn refactor() {
        // args: f: Function, m: Module, s: Selection
        // 

        // let g = fresh function(m)
        // add_function_decl(m, g)
        // let vs = undeclared_vars(f, s)
        // add_params(g, vs)
        // replace_statements_with_call(f, s, g)
    }
}

/*
 * 
01 fn foo() {
02   let mut i = 1;
03   i += 1;
04 }
 * "UndeclaredVariablesVisitor" -- Visits a set of statements. Collects variables appearing "free/unbound" within those statements.
 * "FreshFunctionNameVisitor" -- Returns a fresh name for a function
 * 
 * "ExtractMethodVisitor" -- Given a span (byte start, byte length), it should return the block containing the span and the NodeId's of the first and last statement
 * 
 */