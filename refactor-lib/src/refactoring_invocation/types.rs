use crate::output_types::FileStringReplacement;
use super::*;

pub type QueryResult<T> = Result<T, RefactoringErrorInternal>;

pub enum Query<T> {
    AfterExpansion(Box<dyn Fn(&AstContext) -> QueryResult<T> + Send>),
    AfterParsing(Box<dyn Fn(&TyContext) -> QueryResult<T> + Send>)
}
#[derive(Debug)]
pub struct AstDiff(pub Vec<FileStringReplacement>);
