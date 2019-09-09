use rustc_interface::{interface};
use rustc_driver;
use rustc::hir::intravisit;
use syntax;
use crate::hir_visitor;
use crate::ast_visitor;

pub struct RefactorArgs {
    pub file: String,
    pub method: String,
    pub selection: String
}

pub fn def() -> RefactorArgs {
    RefactorArgs {
        file: "".to_owned(),
        method: "".to_owned(),
        selection: "".to_owned()
    }
}

pub struct MyRefactorCallbacks {
    pub args: RefactorArgs
}

impl rustc_driver::Callbacks for MyRefactorCallbacks {
    // fn after_parsing(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
    //     let sess = compiler.session();
    //     let krate = &compiler.parse().unwrap().peek_mut();
    //     let mut ast_visit = ast_visitor::AstVisitor {
    //         krate
    //     };

    //     syntax::visit::walk_crate(&mut ast_visit, krate);

    //     rustc_driver::Compilation::Continue
    // }
    fn after_expansion(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
        // let sess = compiler.session();
        let krate = &compiler.expansion().unwrap().peek_mut().0;
        let mut ast_visit = ast_visitor::AstVisitor {
            krate
        };

        syntax::visit::walk_crate(&mut ast_visit, krate);

        rustc_driver::Compilation::Continue
    }
    fn after_analysis(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        compiler.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let mut hir_visit = hir_visitor::HirVisitor {
                tcx
            };
            intravisit::walk_crate(&mut hir_visit, tcx.hir().krate());
        });
        rustc_driver::Compilation::Continue
    }
}