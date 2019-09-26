use rustc_interface::{interface};
use rustc_driver;
use rustc::ty;
use crate::refactorings::extract_method;
use crate::refactor_args::RefactorArgs;
use crate::change::Change;


pub struct MyRefactorCallbacks {
    pub args: RefactorArgs,
    pub changes: Vec<Change>
}

impl MyRefactorCallbacks {
    pub fn from_arg(arg: String) -> MyRefactorCallbacks {
        MyRefactorCallbacks {
            args: RefactorArgs::parse(arg),
            changes: vec![]
        }
    }
}

fn do_ty_refactoring(ty: ty::TyCtxt, args: &RefactorArgs) -> Vec<Change> {
    if args.refactoring == "extract-method" {
        extract_method::do_refactoring(ty, args)
    } else {
        vec![]
    }
}

impl rustc_driver::Callbacks for MyRefactorCallbacks {
    // fn after_expansion(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
    //     rustc_driver::Compilation::Continue
    // }
    fn after_analysis(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        compiler.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let changes = do_ty_refactoring(tcx, &self.args);
            
            for change in changes {
                println!("{:?}", change);
            }
        });
        rustc_driver::Compilation::Continue
    }
}
