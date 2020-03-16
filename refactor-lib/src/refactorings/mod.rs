use crate::refactoring_invocation::{FileStringReplacement, RefactoringErrorInternal, RefactorDefinition};
use rustc::ty;

mod box_field;
mod box_named_field;
mod box_tuple_field;
mod close_over_variables;
mod extract_block;
mod inline_macro;
mod introduce_closure;
mod pull_up_item_declaration;
mod split_conflicting_match_arms;
pub mod utils;
pub mod visitors;

pub fn do_ty_refactoring(ty: ty::TyCtxt, args: &RefactorDefinition) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {
    match args {
        RefactorDefinition::BoxField(range) => box_field::do_refactoring(ty, utils::map_range_to_span(&ty.sess.source_map(), range)?),
        RefactorDefinition::CloseOverVariables(range) => close_over_variables::do_refactoring(ty, utils::map_range_to_span(&ty.sess.source_map(), range)?),
        RefactorDefinition::ExtractBlock(range) => extract_block::do_refactoring(ty, utils::map_range_to_span(&ty.sess.source_map(), range)?),
        RefactorDefinition::IntroduceClosure(range) => introduce_closure::do_refactoring(ty, utils::map_range_to_span(&ty.sess.source_map(), range)?),
        RefactorDefinition::SplitConflictingMatchArms(range) => split_conflicting_match_arms::do_refactoring(ty, utils::map_range_to_span(&ty.sess.source_map(), range)?),
        _ => panic!("")
    }
}

pub fn is_after_expansion_refactoring(args: &RefactorDefinition) -> bool {
    if let RefactorDefinition::InlineMacro(..) | RefactorDefinition::PullUpItemDeclaration(..) = args {
        true
    } else {
        false
    }
}
pub fn do_after_expansion_refactoring<'tcx>(queries:  &'tcx rustc_interface::Queries<'tcx>, compiler: &rustc_interface::interface::Compiler, args: &RefactorDefinition) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {

    match args {
        RefactorDefinition::InlineMacro(range) =>{
            let span = utils::map_range_to_span(compiler.session().source_map(), range)?;
            inline_macro::do_refactoring(compiler, &queries, span)
        } ,
        RefactorDefinition::PullUpItemDeclaration(range) => pull_up_item_declaration::do_refactoring(compiler, queries, utils::map_range_to_span(&compiler.source_map(), range)?),
        _ => panic!("")
    }
}