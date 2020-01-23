///
/// Different refactoring definitions with arguments
///
#[derive(PartialEq, Debug)]
pub enum RefactorDefinition {
    ExtractMethod(ExtractMethodArgs),
    ExtractBlock(SourceCodeRange),
    BoxField(SourceCodeRange),
    IntroduceClosure(SourceCodeRange),
    InlineMacro(SourceCodeRange),
    // CloseOverVariables(SourceCodeRange),
    // LiftClosure(SourceCodeRange),
}

#[derive(PartialEq, Debug)]
pub struct ExtractMethodArgs {
    pub range: SourceCodeRange,
    pub new_function: String,
}

///
/// A range in a file. This will later be converted to syntax_pos::Span
/// Note: could be an enum to support different types of ranges (line no, etc)
#[derive(PartialEq, Debug, Clone)]
pub struct SourceCodeRange {
    pub file_name: String,
    pub from: u32,
    pub to: u32,
}

#[derive(Debug, Clone)]
pub struct RefactoringError {
    pub code: InternalErrorCodes,
    pub message: String
}

impl RefactoringError {
    pub fn new(code: InternalErrorCodes, message: String) -> Self {
        Self { code, message }
    }
    pub fn used_in_pattern(ident: &str) -> Self {
        Self::new(InternalErrorCodes::Error,
            format!(
                "Field: {} is used in a pattern and cannot be boxed.",
                ident))
    }
    pub fn file_not_found(name: &str) -> Self {
        Self::new(InternalErrorCodes::FileNotFound,
            format!(
                "Couldn't find file: {}",
                name))
    }
    pub fn invalid_selection(from: u32, to: u32) -> Self {
        Self::new(InternalErrorCodes::Error,
            format!(
                "{}:{} is not a valid selection!",
                from, to))
    }
    pub fn invalid_selection_with_code(from: u32, to: u32, selection: &str) -> Self {
        Self::new(InternalErrorCodes::Error,
            format!(
                "{}:{} is not a valid selection! `{}`",
                from, to, selection))
    }
    pub fn multiple_returnvalues() -> Self {
        Self::new(InternalErrorCodes::Error, 
                "Multiple returnvalues not implemented".to_owned())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum InternalErrorCodes {
    Error = 0,
    FileNotFound = 1
}