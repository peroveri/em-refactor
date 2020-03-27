use crate::output_types::FileStringReplacement;
use super::*;
pub trait AfterExpansionRefactoring {
    fn refactor(&self, ast: &AstContext) -> QueryResult<AstDiff>;
}
pub trait AfterExpansion {
    fn after_expansion(&self, ast: &AstContext) -> QueryResult<String>;
}

pub type QueryResult<T> = Result<T, RefactoringErrorInternal>;
// pub struct TyContext {}

pub enum Query {
    AfterExpansion(Box<dyn AfterExpansion + Send>)
    // AfterExpansion(dyn Refactoring)
}
#[derive(Debug)]
pub struct AstDiff(pub Vec<FileStringReplacement>);

pub struct OutputMain {
    refa: Box<dyn AfterExpansionRefactoring + Send>
}

impl OutputMain {
    pub fn new(refa: Box<dyn AfterExpansionRefactoring + Send>) -> Self {
        Self {
            refa
        }
    }
}
impl AfterExpansion for OutputMain {
    fn after_expansion(&self, ast: &AstContext) -> QueryResult<String> {
        let r = self.refa.refactor(ast)?;
        let content = get_file_content(&r.0, ast.get_source_map());

        content.ok_or_else(|| RefactoringErrorInternal::file_not_found("todo!"))
    }
}
pub struct ToJson {
    refa: Box<dyn AfterExpansionRefactoring + Send>
}
impl ToJson {
    pub fn new(refa: Box<dyn AfterExpansionRefactoring + Send>) -> Self {
        Self {
            refa
        }
    }
}

impl AfterExpansion for ToJson {
    fn after_expansion(&self, ast: &AstContext) -> QueryResult<String> {
        self.refa.refactor(ast).map(|e| format!("{}", e.0.len()))
    }
}