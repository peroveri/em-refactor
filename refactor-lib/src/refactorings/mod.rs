use rustc::ty;
use crate::my_refactor_callbacks::RefactorArgs;

// mod extract_method;
mod hir_visitor;
pub mod extract_method;

// pub fn refactor<'v>(tcx: &'v ty::TyCtxt<'v>, args: &RefactorArgs) {
//     extract_method::extract_method(tcx, "", (31, 12));
//     // let mut hir_visit = hir_visitor::StmtsVisitor {
//     //     tcx,
//     //     span: 
//     // };

//     // intravisit::walk_crate(&mut hir_visit, tcx.hir().krate());
// }
