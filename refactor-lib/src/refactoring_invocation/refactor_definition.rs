///
/// Different refactoring definitions with arguments
///
#[derive(PartialEq, Debug)]
pub enum RefactorDefinition {
    ExtractBlock(SourceCodeRange),
    BoxField(SourceCodeRange),
    CloseOverVariables(SourceCodeRange),
    IntroduceClosure(SourceCodeRange),
    InlineMacro(SourceCodeRange),
    PullUpItemDeclaration(SourceCodeRange),
    SplitConflictingMatchArms(SourceCodeRange),
    // CloseOverVariables(SourceCodeRange),
    // LiftClosure(SourceCodeRange),
}

// refactoring result pr crate
// - crash (bad format on input, didnt compile, unhandled error, ++) => stop execution
// - not applicable (didnt find file or ast-node, refactoring is not possible, refactoring is not safe)
// - found valid refactoring (list of changes)

// maybe generic implementation of rustc_driver::Callbacks?

///
/// A range in a file. This will later be converted to syntax_pos::Span
/// Note: could be an enum to support different types of ranges (line no, etc)
#[derive(PartialEq, Debug, Clone)]
pub struct SourceCodeRange {
    pub file_name: String,
    pub from: u32,
    pub to: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefactoringErrorInternal {
    pub code: InternalErrorCodes,
    pub message: String
}

// Errors that should cause the tool to halt
// this should erplace the other error types
#[derive(Debug, PartialEq)]
pub enum RefactorFailCode {
    BadFormatOnInput = 1,
    CompilerErr = 2,
    InternalRefactoringError = 3,
}

#[derive(Debug, PartialEq)]
pub struct RefactorFail {
    pub code: RefactorFailCode,
    pub message: String,
}

impl RefactorFail {
    pub fn arg_def(s: &str) -> Self {
        Self {
            code: RefactorFailCode::BadFormatOnInput,
            message: s.to_string(),
        }
    }
    pub fn compile_err() -> Self {
        Self {
            code: RefactorFailCode::CompilerErr,
            message: "failed during refactoring".to_string(),
        }
    }
    pub fn int(s: &str) -> Self {
        Self {
            code: RefactorFailCode::InternalRefactoringError,
            message: s.to_string(),
        }
    }
}

impl RefactoringErrorInternal {
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum InternalErrorCodes {
    Error = 0,
    FileNotFound = 1
}