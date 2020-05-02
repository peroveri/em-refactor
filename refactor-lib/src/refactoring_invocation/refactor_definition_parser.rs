use crate::refactoring_invocation::{AstContext, Query, QueryResult, RefactoringErrorInternal};
use crate::refactorings::{box_field, close_over_variables, convert_closure_to_fn, extract_block, inline_macro, introduce_closure, pull_up_item_declaration, remove_refactoring_comments};
use crate::refactoring_invocation::{AstDiff, TyContext};
use refactor_lib_types::{defs::*, RefactorArgs};
use rustc_span::Span;
///
/// converts an argument list to a refactoring definition
///
pub fn argument_list_to_refactor_def(args: RefactorArgs) -> QueryResult<Query<AstDiff>> {
    match args.refactoring.as_ref() {
        BOX_FIELD => Ok(to_ty_query(args, Box::new(box_field::do_refactoring))),
        CLOSE_OVER_VARIABLES => Ok(to_ty_query(args, Box::new(close_over_variables::do_refactoring))),
        CONVERT_CLOSURE_TO_FUNCTION => Ok(to_ty_query(args, Box::new(convert_closure_to_fn::do_refactoring))),
        EXTRACT_BLOCK => Ok(to_ty_query(args, Box::new(extract_block::do_refactoring))),
        INTRODUCE_CLOSURE => Ok(to_ty_query(args, Box::new(introduce_closure::do_refactoring))),
        "inline-macro" => Ok(to_ast_query(args, Box::new(inline_macro::do_refactoring))),
        PULL_UP_ITEM_DECLARATIONS => Ok(to_ast_query(args, Box::new(pull_up_item_declaration::do_refactoring))),
        REMOVE_REFACTORING_COMMENTS => Ok(to_ast_query(args, Box::new(remove_refactoring_comments::do_refactoring))),
        s => Err(RefactoringErrorInternal::arg_def(&format!("Unknown refactoring: {}", s)))
    }
}

fn to_ast_query(args: RefactorArgs, f: Box<dyn Fn(&AstContext, Span, bool) -> QueryResult<AstDiff> + Send>) -> Query<AstDiff> {
    let args = args.clone();
    Query::AfterExpansion(Box::new(move |ast| {
        let span = ast.source().map_selection_to_span(args.selection.clone(), args.file.clone())?;
        f(ast, span, args.add_comment)
    }))
}

fn to_ty_query(args: RefactorArgs, f: Box<dyn Fn(&TyContext, Span, bool) -> QueryResult<AstDiff> + Send>) -> Query<AstDiff> {
    Query::AfterParsing(Box::new(move |ast| {
        let span = ast.source().map_selection_to_span(args.selection.clone(), args.file.clone())?;
        f(ast, span, args.add_comment)
    }))
}
