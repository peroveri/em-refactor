use super::arg_value;
use rustc_interface::interface;
use syntax::visit::{Visitor, walk_crate};
use syntax::ast::Block;

pub fn should_query_candidates(refactor_args: &[String]) -> bool {
    arg_value(refactor_args, "--query-candidates", |_| true).is_some()
}

pub fn list_candidates(refactor_args: &[String], rustc_args: &[String]) -> Result<(), i32> {

    let c = arg_value(refactor_args, "--query-candidates", |_| true).unwrap();

    let mut callbacks = RustcAfterParsing(c.to_string());

    let emitter = Box::new(Vec::new());
    std::env::set_var("RUST_BACKTRACE", "1");
    let err = rustc_driver::run_compiler(&rustc_args, &mut callbacks, None, Some(emitter));
    err.unwrap();
    Ok(())
}


struct RustcAfterParsing(String); // TODO: after parsing or expansion?

impl rustc_driver::Callbacks for RustcAfterParsing
{
    fn after_parsing<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {

        let candidates = collect_extract_block_cand(queries);



        print!("[{}]", candidates.iter().map(|t| format!("{}, {}", t.0, t.1)).collect::<Vec<_>>().join(","));

        rustc_driver::Compilation::Stop
    }
}

fn collect_extract_block_cand<'tcx>(queries: &'tcx rustc_interface::Queries<'tcx>) -> Vec<(u32, u32)> {
    let mut v = CandVisitor{candidates: vec![]};

    let crate_ = &*queries.parse().unwrap().peek_mut();

    walk_crate(&mut v, crate_);

    v.candidates
}


struct CandVisitor {
    candidates: Vec<(u32, u32)>
}

impl<'ast> Visitor<'ast> for CandVisitor {
    fn visit_block(&mut self, b: &'ast Block) {
        self.candidates.push((b.span.lo().0, b.span.hi().0));
    }
}