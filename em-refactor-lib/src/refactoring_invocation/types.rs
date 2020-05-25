use em_refactor_lib_types::FileStringReplacement;
use super::{AstContext, RefactoringErrorInternal, TyContext};

pub type QueryResult<T> = Result<T, RefactoringErrorInternal>;
// pub type AstRefactoring = fn(&AstContext, Span, bool) -> QueryResult<AstDiff>;
// pub type TyRefactoring = fn(&TyContext, Span, bool) -> QueryResult<AstDiff>;

pub enum Query<T> {
    AfterExpansion(Box<dyn Fn(&AstContext) -> QueryResult<T> + Send>),
    AfterParsing(Box<dyn Fn(&TyContext) -> QueryResult<T> + Send>)
}
#[derive(Debug)]
pub struct AstDiff(pub Vec<FileStringReplacement>);
