use rustc_interface::{interface};
use rustc_driver;
use rustc::hir::intravisit;
use syntax;

use crate::refactorings;

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

pub struct Change {
    file_name: String,
    start: u32,
    end: u32,
    replacement: String
}

pub struct MyRefactorCallbacks {
    pub args: RefactorArgs,
    pub changes: Vec<Change>
}

fn extract_path(fname: syntax::source_map::FileName) -> Option<std::path::PathBuf> {
    if let syntax::source_map::FileName::Real(p) = fname {
        return Some(p);
    }
    return None;
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
        // let krate = &compiler.expansion().unwrap().peek_mut().0;
        // let source_map = &compiler.source_map();
        // let mut ast_visit = ast_visitor::AstVisitor {
        //     krate,
        //     source_map
        // };

        // syntax::visit::walk_crate(&mut ast_visit, krate);

        rustc_driver::Compilation::Continue
    }
    fn after_analysis(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        compiler.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let mut extract_method = refactorings::extract_method::ExtractMethodRefactoring {
                tcx,
            };
            
            if let Some(span) = extract_method.extract_method("", (35, 42)) {
                let source = compiler.source_map();
                let body = compiler.source_map().span_to_snippet(span).unwrap();

                let new_fn = format!("fn new_fn() {{ {} }}", body);

                eprintln!("{}", new_fn);
                eprintln!("{:?}", source.span_to_filename(span));
                eprintln!("{:?}", extract_path(source.span_to_filename(span)).unwrap().as_os_str());
                // self.changes.push(Change {
                //     file_name: source.span_to_filename(span),
                //     start: 0,
                //     end: 0,
                //     replacement: new_fn
                // });
            }

            // refactorings::refactor(&tcx, &self.args);
        });
        rustc_driver::Compilation::Continue
    }
}