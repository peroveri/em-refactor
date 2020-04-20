use crate::refactoring_invocation::{AstContext, Query, QueryResult, RefactoringErrorInternal, SourceCodeRange};
use crate::refactorings::{box_field, close_over_variables, convert_closure_to_fn, extract_block, inline_macro, introduce_closure, pull_up_item_declaration, split_conflicting_match_arms};
use crate::refactoring_invocation::{AstDiff, TyContext};
use rustc_span::Span;
///
/// converts an argument list to a refactoring definition
///
pub fn argument_list_to_refactor_def(args: &[String]) -> Result<Query<AstDiff>, RefactoringErrorInternal> {
    let parser = RefactorArgsParser { args };
    let args = parser.from_args()?;
    map_args_to_query(args)
}

struct RefactorArgsParser<'a> {
    args: &'a [String],
}
#[derive(Debug, PartialEq)]
struct RefactoringArgs {
    refactoring: String,
    range: SourceCodeRange
}
fn to_ast_query(r: SourceCodeRange, f: Box<dyn Fn(&AstContext, Span) -> QueryResult<AstDiff> + Send>) -> Query<AstDiff> {
    Query::AfterExpansion(Box::new(move |ast| {
        f(ast, ast.map_range_to_span(&r)?)
    }))
}
fn to_ty_query(r: SourceCodeRange, f: Box<dyn Fn(&TyContext, Span) -> QueryResult<AstDiff> + Send>) -> Query<AstDiff> {
    Query::AfterParsing(Box::new(move |ast| {
        f(ast, ast.map_range_to_span(&r)?)
    }))
}

fn map_args_to_query(args: RefactoringArgs) -> Result<Query<AstDiff>, RefactoringErrorInternal> {
    match args.refactoring.as_ref() {
        "box-field" => Ok( to_ty_query(args.range, Box::new(box_field::do_refactoring))),
        "close-over-variables" => Ok(to_ty_query(args.range, Box::new(close_over_variables::do_refactoring))),
        "convert-closure-to-function" => Ok(to_ty_query(args.range, Box::new(convert_closure_to_fn::do_refactoring))),
        "extract-block" => Ok(to_ty_query(args.range, Box::new(extract_block::do_refactoring))),
        "introduce-closure" => Ok(to_ty_query(args.range, Box::new(introduce_closure::do_refactoring))),
        "inline-macro" => Ok(to_ast_query(args.range, Box::new(inline_macro::do_refactoring))),
        "pull-up-item-declaration" => Ok(to_ast_query(args.range, Box::new(pull_up_item_declaration::do_refactoring))),
        "split-conflicting-match-arms" => Ok(to_ty_query(args.range, Box::new(split_conflicting_match_arms::do_refactoring))),
        s => Err(RefactoringErrorInternal::arg_def(&format!("Unknown refactoring: {}", s)))
    }
}
impl RefactorArgsParser<'_> {
    pub fn from_args(&self) -> Result<RefactoringArgs, RefactoringErrorInternal> {
        Ok(RefactoringArgs {
            range: self.parse_range()?,
            refactoring: self.get_param("--refactoring")?.to_string()
        })
    }
    pub fn parse_range(&self) -> Result<SourceCodeRange, RefactoringErrorInternal> {
        let selection = self.get_param("--selection")?;
        let file = self.get_param("--file")?;
        let ints = Self::get_int(selection)?;

        Ok(SourceCodeRange {
            file_name: file.to_string(),
            from: ints.0,
            to: ints.1,
        })
    }
    pub fn get_int(selection: &str) -> Result<(u32, u32), RefactoringErrorInternal> {
        let mut split = selection.split(':');
        if let (Some(from), Some(to)) = (split.nth(0), split.nth(0)) {
            let from = from.parse().map_err(|_| RefactoringErrorInternal::arg_def(&format!("{} is not a valid int", from)))?;
            let to = to.parse().map_err(|_| RefactoringErrorInternal::arg_def(&format!("{} is not a valid int", from)))?;
            return Ok((from, to));
        }
        Err(RefactoringErrorInternal::arg_def("Selection should be formatted as <byte_from>:<byte_to>"))
    }
    fn get_param(&self, name: &str) -> Result<&str, RefactoringErrorInternal> {
        for t in self.args {
            let mut s = t.split('=');
            if s.nth(0) == Some(name) {
                if let Some(r) = s.nth(0) {
                    return Ok(r);
                }
            }
        }
        Err(RefactoringErrorInternal::arg_def(&format!("Expected {}", name)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn refactor_def_from_args() {
        let parser = RefactorArgsParser {
            args: &vec![
                "--refactoring=extract-block".to_owned(),
                "--file=main.rs".to_owned(),
                "--selection=1:2".to_owned(),
            ]
        };
        let expected = Ok(RefactoringArgs {
            range: SourceCodeRange {
                from: 1,
                to: 2,
                file_name: "main.rs".to_owned(),
            },
            refactoring: "extract-block".to_string()
        });

        let actual = parser.from_args();

        assert_eq!(expected, actual);
    }
}
