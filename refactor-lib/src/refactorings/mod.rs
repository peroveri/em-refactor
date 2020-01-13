use crate::change::Change;
use crate::refactor_definition::{RefactoringError, RefactorDefinition};
use rustc::ty;

mod box_field;
mod extract_block;
mod extract_method;
mod introduce_closure;
pub mod utils;

pub fn do_ty_refactoring(ty: ty::TyCtxt, args: &RefactorDefinition) -> Result<Vec<Change>, RefactoringError> {
    match args {
        RefactorDefinition::BoxField(range) => box_field::do_refactoring(ty, utils::map_range_to_span(ty, range)?),
        RefactorDefinition::ExtractMethod(args) => {
            extract_method::do_refactoring(ty, &args.range, &args.new_function)
        }
        RefactorDefinition::ExtractBlock(range) => extract_block::do_refactoring(ty, utils::map_range_to_span(ty, range)?),
        RefactorDefinition::IntroduceClosure(range) => introduce_closure::do_refactoring(ty, utils::map_range_to_span(ty, range)?),
    }
}
