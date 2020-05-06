use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileStringReplacement {
    pub file_name: String,
    pub line_start: usize,
    pub char_start: usize,
    pub line_end: usize,
    pub char_end: usize,
    pub byte_start: u32,
    pub byte_end: u32,
    pub replacement: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RefactoringError {
    pub is_error: bool,
    pub kind: RefactorErrorType,
    pub message: String,
    pub codes: Vec<String>,
    pub at_refactoring: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RefactorErrorType {
    Internal = 0,
    /// The initial compilation failed
    RustCError1 = 1,
    /// The compile check after refactoring failed
    RustCError2 = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactorOutput {
    pub crate_name: String,
    pub is_test: bool,
    pub replacements: Vec<FileStringReplacement>,
    pub errors: Vec<RefactoringError>
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactorOutputs { // Single
    pub candidates: Vec<CandidateOutput>,
    pub refactorings: Vec<RefactorOutput>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RefactorOutputs2 {
    pub candidates: Vec<CandidatePosition>,
    pub changes: Vec<Vec<FileStringReplacement>>,
    pub errors: Vec<RefactoringError> // Map<Crate, Err[]>?
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateOutput {
    pub candidates: Vec<CandidatePosition>,
    pub crate_name: String,
    pub is_test: bool,
    pub refactoring: String,
    pub errors: Vec<RefactoringError>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CandidatePosition {
    pub file: String,
    pub from: u32,
    pub to: u32,
    pub nrof_lines: Option<u32>
}

impl CandidatePosition {
    pub fn new(file: &str, from: u32, to: u32, nrof_lines: Option<u32>) -> Self {
        Self {
            file: file.to_string(),
            from,
            to,
            nrof_lines
        }
    }
}

impl RefactorOutputs2 {
    pub fn empty() -> Self {
        Self::new(vec![], vec![], vec![])
    }
    pub fn new(candidates: Vec<CandidatePosition>, changes: Vec<Vec<FileStringReplacement>>, errors: Vec<RefactoringError>) -> Self {
        Self {
            candidates,
            changes,
            errors
        }
    }
    pub fn from_error(error: RefactoringError) -> Self {
        Self::from_errors(vec![error])
    }
    pub fn from_errors(errors: Vec<RefactoringError>) -> Self {
        Self::new(vec![], vec![], errors)
    }
    pub fn from_candidates(candidates: Vec<CandidatePosition>) -> Self {
        Self::new(candidates, vec![], vec![])
    }
    pub fn from_change(change: FileStringReplacement) -> Self {
        Self::from_changes(vec![change])
    }
    pub fn from_changes(changes: Vec<FileStringReplacement>) -> Self {
        Self::new(vec![], vec![changes], vec![])
    }
}

impl RefactorOutputs {
    pub fn new() -> Self {
        Self {
            candidates: vec![],
            refactorings: vec![]
        }
    }
    pub fn from_candidate(candidate: CandidateOutput) -> Self {
        Self {
            candidates: vec![candidate],
            refactorings: vec![]
        }
    }
    pub fn from_candidates(candidates: Vec<CandidateOutput>) -> Self {
        Self {
            candidates,
            refactorings: vec![]
        }
    }
    pub fn from_refactorings(refactorings: Vec<RefactorOutput>) -> Self {
        Self {
            candidates: vec![],
            refactorings
        }
    }
    #[allow(unused)]
    pub fn sort(&mut self) {
        self.candidates.sort_by_key(|a| (a.crate_name.clone(), a.is_test));
        self.refactorings.sort_by_key(|a| (a.crate_name.clone(), a.is_test));
    }
    #[allow(unused)]
    pub fn extend(&mut self, other: RefactorOutputs) {
        self.candidates.extend(other.candidates);
        self.refactorings.extend(other.refactorings);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SelectionType {
    Range(String),
    Comment(String)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactorArgs {
    pub refactoring: String,
    pub selection: SelectionType,
    pub file: String,
    pub unsafe_: bool,
    pub deps: Vec<String>,
    pub add_comment: bool,
    pub with_changes: Vec<Vec<FileStringReplacement>>
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateArgs {
    pub refactoring: String,
    pub deps: Vec<String>
}

pub fn create_refactor_tool_marker(item: &str, end: bool) -> String {
    format!("/*{}:{}:{}*/", defs::REFACTOR_TOOL_MARKER, item, if end {"end"} else {"start"})
}

pub mod defs {
    pub const BOX_FIELD: &str = "box-field";
    pub const BOX_FIELD_CANDIDATES: &str = BOX_FIELD;
    pub const CLOSE_OVER_VARIABLES: &str = "close-over-variables";
    pub const CONVERT_CLOSURE_TO_FUNCTION: &str = "convert-closure-to-function";
    pub const EXTRACT_BLOCK: &str = "extract-block";
    pub const EXTRACT_BLOCK_BLOCK: &str = "extract-block.block";
    pub const EXTRACT_METHOD: &str = "extract-method";
    pub const EXTRACT_METHOD_CANDIDATES: &str = EXTRACT_METHOD;
    pub const INTRODUCE_CLOSURE: &str = "introduce-closure";
    pub const INTRODUCE_CLOSURE_CALL_EXPR: &str = "introduce-closure.call-expr";
    pub const PULL_UP_ITEM_DECLARATIONS: &str = "pull-up-item-declaration";
    pub const PULL_UP_ITEM_DECLARATIONS_STMTS: &str = "pull-up-item-declaration.stmts";
    pub const REMOVE_REFACTORING_COMMENTS: &str = "remove-refactoring-comments";
    pub const REFACTOR_TOOL_MARKER: &str = "refactor-tool";
    pub const ENV_REFACTORING_ARGS: &str = "REFACTORING_ARGS";
    pub const ENV_CANDIDATE_ARGS: &str = "CANDIDATE_ARGS";

    pub fn extract_method_def() -> Vec<(&'static str, &'static str)> {
        vec![
            (PULL_UP_ITEM_DECLARATIONS, ""),
            (EXTRACT_BLOCK, PULL_UP_ITEM_DECLARATIONS_STMTS),
            (INTRODUCE_CLOSURE, EXTRACT_BLOCK_BLOCK),
            (CLOSE_OVER_VARIABLES, INTRODUCE_CLOSURE_CALL_EXPR),
            (CONVERT_CLOSURE_TO_FUNCTION, INTRODUCE_CLOSURE_CALL_EXPR),
            (REMOVE_REFACTORING_COMMENTS, ""),
        ]
    }
}