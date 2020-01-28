use crate::refactoring_invocation::{FileReplaceContent, RefactoringErrorInternal, RefactorDefinition};
use rustc::ty;

mod box_field;
mod box_named_field;
mod box_tuple_field;
mod extract_block;
mod extract_method;
mod inline_macro;
mod introduce_closure;
pub mod utils;
pub mod visitors;

pub fn do_ty_refactoring(ty: ty::TyCtxt, args: &RefactorDefinition) -> Result<Vec<FileReplaceContent>, RefactoringErrorInternal> {
    match args {
        RefactorDefinition::BoxField(range) => box_field::do_refactoring(ty, utils::map_range_to_span(&ty.sess.source_map(), range)?),
        RefactorDefinition::ExtractMethod(args) => {
            extract_method::do_refactoring(ty, &args.range, &args.new_function)
        }
        RefactorDefinition::ExtractBlock(range) => extract_block::do_refactoring(ty, utils::map_range_to_span(&ty.sess.source_map(), range)?),
        RefactorDefinition::IntroduceClosure(range) => introduce_closure::do_refactoring(ty, utils::map_range_to_span(&ty.sess.source_map(), range)?),
        _ => panic!("")
    }
}

pub fn is_after_expansion_refactoring(args: &RefactorDefinition) -> bool {
    if let RefactorDefinition::InlineMacro(..) = args {
        true
    } else {
        false
    }
}
pub fn do_after_expansion_refactoring<'tcx>(queries:  &'tcx rustc_interface::Queries<'tcx>, compiler: &rustc_interface::interface::Compiler, args: &RefactorDefinition) -> Result<Vec<FileReplaceContent>, RefactoringErrorInternal> {

    match args {
        RefactorDefinition::InlineMacro(range) =>{
            let span = utils::map_range_to_span(compiler.session().source_map(), range).unwrap();
            inline_macro::do_refactoring(compiler, &queries, span)
        } ,
        _ => panic!("")
    }
}