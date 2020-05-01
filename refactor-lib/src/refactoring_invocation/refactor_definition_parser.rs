use crate::refactoring_invocation::{AstContext, Query, QueryResult, RefactoringErrorInternal};
use crate::refactorings::{box_field, close_over_variables, convert_closure_to_fn, extract_block, inline_macro, introduce_closure, pull_up_item_declaration, split_conflicting_match_arms};
use crate::refactoring_invocation::{AstDiff, TyContext};
use refactor_lib_types::RefactorArgs;
use rustc_span::Span;
///
/// converts an argument list to a refactoring definition
///
pub fn argument_list_to_refactor_def(args: RefactorArgs) -> QueryResult<Query<AstDiff>> {
    match args.refactoring.as_ref() {
        "box-field" => Ok(to_ty_query(args, Box::new(box_field::do_refactoring))),
        "close-over-variables" => Ok(to_ty_query(args, Box::new(close_over_variables::do_refactoring))),
        "convert-closure-to-function" => Ok(to_ty_query(args, Box::new(convert_closure_to_fn::do_refactoring))),
        "extract-block" => Ok(to_ty_query(args, Box::new(extract_block::do_refactoring))),
        "introduce-closure" => Ok(to_ty_query(args, Box::new(introduce_closure::do_refactoring))),
        "inline-macro" => Ok(to_ast_query(args, Box::new(inline_macro::do_refactoring))),
        "pull-up-item-declaration" => Ok(to_ast_query(args, Box::new(pull_up_item_declaration::do_refactoring))),
        "split-conflicting-match-arms" => Ok(to_ty_query(args, Box::new(split_conflicting_match_arms::do_refactoring))),
        s => Err(RefactoringErrorInternal::arg_def(&format!("Unknown refactoring: {}", s)))
    }
}

fn to_ast_query(args: RefactorArgs, f: Box<dyn Fn(&AstContext, Span) -> QueryResult<AstDiff> + Send>) -> Query<AstDiff> {
    let args = args.clone();
    Query::AfterExpansion(Box::new(move |ast| {
        let span = ast.source().map_selection_to_span(args.selection.clone(), args.file.clone())?;
        f(ast, span)
    }))
}

fn to_ty_query(args: RefactorArgs, f: Box<dyn Fn(&TyContext, Span) -> QueryResult<AstDiff> + Send>) -> Query<AstDiff> {
    Query::AfterParsing(Box::new(move |ast| {
        let span = ast.source().map_selection_to_span(args.selection.clone(), args.file.clone())?;
        f(ast, span)
    }))
}
