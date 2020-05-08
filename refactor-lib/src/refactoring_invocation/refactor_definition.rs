use refactor_lib_types::RefactorErrorType;

// refactoring result pr crate
// - crash (bad format on input, didnt compile, unhandled error, ++) => stop execution
// - not applicable (didnt find file or ast-node, refactoring is not possible, refactoring is not safe)
// - found valid refactoring (list of changes)

// maybe generic implementation of rustc_driver::Callbacks?

#[derive(Debug, Clone, PartialEq)]
pub struct RefactoringErrorInternal {
    pub error_type: RefactorErrorType,
    pub is_error: bool,
    pub message: String,
    pub external_codes: Vec<String>
}

impl RefactoringErrorInternal {
    fn new(error_type: RefactorErrorType, is_error: bool, message: String, external_codes: Vec<String>) -> Self {
        Self { error_type, is_error, message, external_codes }
    }
    fn new_int_soft(code: InternalErrorCodes, message: String) -> Self {
        Self::new(RefactorErrorType::Internal, false, message, vec![format!("{:?}", code)])
    }
    fn new_int(code: InternalErrorCodes, message: String) -> Self {
        Self::new(RefactorErrorType::Internal, true, message, vec![format!("{:?}", code)])
    }
    pub fn used_in_pattern(ident: &str) -> Self {
        Self::new_int(InternalErrorCodes::Error,
            format!(
                "Field: {} is used in a pattern and cannot be boxed.",
                ident))
    }
    pub fn comment_not_found(name: &str) -> Self {
        Self::new_int_soft(InternalErrorCodes::FileNotFound,
            format!(
                "Couldn't find comment: {}",
                name))
    }
    pub fn file_not_found(name: &str) -> Self {
        Self::new_int(InternalErrorCodes::FileNotFound,
            format!(
                "Couldn't find file: {}",
                name))
    }
    pub fn file_not_found_soft(name: &str) -> Self {
        Self::new_int_soft(InternalErrorCodes::FileNotFound,
            format!(
                "Couldn't find file: {}",
                name))
    }
    pub fn invalid_argument(msg: String) -> Self {
        Self::new_int(InternalErrorCodes::Error,
            msg)
    }
    pub fn invalid_selection(from: u32, to: u32) -> Self {
        Self::new_int(InternalErrorCodes::InvalidSelection,
            format!(
                "{}:{} is not a valid selection!",
                from, to))
    }
    pub fn invalid_selection_with_code(from: u32, to: u32, selection: &str) -> Self {
        Self::new_int(InternalErrorCodes::InvalidSelection,
            format!(
                "{}:{} is not a valid selection! `{}`",
                from, to, selection))
    }
    pub fn refactoring_not_invoked() -> Self {
        Self::new_int(InternalErrorCodes::Error,
           "The refactoring was not invoked".to_owned())
    }
    pub fn int(s: &str) -> Self {
        Self::new_int(InternalErrorCodes::Internal, s.to_string())
    }
    pub fn arg_def(s: &str) -> Self {
        Self::new_int(InternalErrorCodes::BadFormatOnInput, s.to_string())
    }
    pub fn compile_err() -> Self {
        Self::new(RefactorErrorType::RustCError1, true, "Compile err".to_owned(), vec![])
    }
    pub fn recompile_err(s: &str, codes: Vec<String>) -> Self {
        Self::new(RefactorErrorType::RustCError2, true, s.to_string(), codes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InternalErrorCodes {
    Error = 0,
    FileNotFound = 1,
    Internal = 2,
    BadFormatOnInput = 3,
    InvalidSelection = 4,
}