///
/// Different refactoring definitions with arguments
///
#[derive(PartialEq, Debug)]
pub enum RefactorDefinition {
    ExtractMethod(ExtractMethodArgs),
    ExtractBlock(SourceCodeRange),
    // IntroduceClosure(SourceCodeRange),
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
#[derive(PartialEq, Debug)]
pub struct SourceCodeRange {
    pub file_name: String,
    pub from: u32,
    pub to: u32,
}
