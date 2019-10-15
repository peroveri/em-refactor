use crate::change::Change;
use crate::refactor_definition::RefactorDefinition;
use rustc::ty;

mod extract_block;
mod extract_method;

pub fn do_ty_refactoring(ty: ty::TyCtxt, args: &RefactorDefinition) -> Result<Vec<Change>, String> {
    match args {
        RefactorDefinition::ExtractMethod(args) => {
            extract_method::do_refactoring(ty, &args.range, &args.new_function)
        }
        RefactorDefinition::ExtractBlock(range) => extract_block::do_refactoring(ty, &range),
    }
}
