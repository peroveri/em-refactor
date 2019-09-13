use rustc_interface::interface;
use rustc_driver;
use refactor_lib::{RefactorArgs, refactor};

pub struct ClippyCallbacks {
    pub args: RefactorArgs
}

impl rustc_driver::Callbacks for ClippyCallbacks {
    fn after_parsing(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
        let sess = compiler.session();
        let mut registry = rustc_driver::plugin::registry::Registry::new(
            sess,
            compiler
                .parse()
                .expect(
                    "at this compilation stage \
                     the crate must be parsed",
                )
                .peek()
                .span,
        );
        println!("parse");
        // registry.args_hidden = Some(Vec::new());

        // let conf = clippy_lints::read_conf(&registry);
        // register_lints(&mut registry);

        // let rustc_driver::plugin::registry::Registry {
        //     early_lint_passes,
        //     late_lint_passes,
        //     lint_groups,
        //     llvm_passes,
        //     attributes,
        //     ..
        // } = registry;
        // let mut ls = sess.lint_store.borrow_mut();
        // for pass in early_lint_passes {
        //     ls.register_early_pass(Some(sess), true, false, pass);
        // }
        // for pass in late_lint_passes {
        //     ls.register_late_pass(Some(sess), true, false, false, pass);
        // }

        // for (name, (to, deprecated_name)) in lint_groups {
        //     ls.register_group(Some(sess), true, name, deprecated_name, to);
        // }
        // clippy_lints::register_pre_expansion_lints(sess, &mut ls, &conf);
        // clippy_lints::register_renamed(&mut ls);

        // sess.plugin_llvm_passes.borrow_mut().extend(llvm_passes);
        // sess.plugin_attributes.borrow_mut().extend(attributes);

        // my_mut_visit::mut_visit();

        refactor(&self.args);


        // Continue execution
        rustc_driver::Compilation::Continue
    }
    fn after_analysis(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        compiler.global_ctxt().unwrap().peek_mut().enter(|_tcx| {
            // my_visit::find_candidates(tcx);
        });
        rustc_driver::Compilation::Continue
    }
}