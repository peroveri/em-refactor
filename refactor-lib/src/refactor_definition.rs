///
/// Different refactoring definitions with arguments
///
#[derive(PartialEq, Debug)]
pub enum RefactorDefinition {
    ExtractMethod(ExtractMethodArgs),
    ExtractBlock(SourceCodeRange),
    BoxField(SourceCodeRange),
    IntroduceClosure(SourceCodeRange),
    // CloseOverVariables(SourceCodeRange),
    // LiftClosure(SourceCodeRange),
}

// plan for compositions:
// ExtractBlock            = InsertEmptyBlock     o PushExpression     o PushStatements
// ExtractClosure          = ExtractBlock         o IntroduceClosure
// ExtractLocalFunction    = ExtractClosure       o CloseOverVariables o IntroduceLocalFunction
// ExtractMethod /Function = ExtractLocalFunction o LiftLocalFunction

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
