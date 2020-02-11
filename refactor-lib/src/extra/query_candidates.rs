use super::{arg_value, collect_extract_block_candidates, collect_box_field_candidates, CollectFieldMode, CandidateOutput, CandidatePosition};
use rustc_interface::interface;
use rustc_span::Span;
use crate::refactorings::utils::map_span_to_index;

pub fn should_query_candidates(refactor_args: &[String]) -> bool {
    arg_value(refactor_args, "--query-candidates", |_| true).is_some()
}

pub fn list_candidates(refactor_args: &[String], rustc_args: &[String]) -> Result<(), i32> {

    let c = arg_value(refactor_args, "--query-candidates", |_| true).unwrap();

    let crate_name = arg_value(rustc_args, "--crate-name", |_| true).unwrap();

    let mut callbacks = RustcAfterParsing(c.to_string(), crate_name.to_string());

    let emitter = Box::new(Vec::new());
    std::env::set_var("RUST_BACKTRACE", "1");
    let err = rustc_driver::run_compiler(&rustc_args, &mut callbacks, None, Some(emitter));
    err.unwrap();
    Ok(())
}


struct RustcAfterParsing(String, String); // TODO: after parsing or expansion?

impl rustc_driver::Callbacks for RustcAfterParsing
{
    fn after_parsing<'tcx>(
        &mut self,
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {

        let candidates = 
        match self.0.as_ref() {
            "extract-block" => collect_extract_block_candidates(queries),
            "box-field" => collect_box_field_candidates(queries, CollectFieldMode::All),
            "box-named-field" => collect_box_field_candidates(queries, CollectFieldMode::Named),
            "box-tuple-field" => collect_box_field_candidates(queries, CollectFieldMode::Tuple),
            _ => panic!("Unknown argument to query-candidate: `{}`", self.0)
        };

        print_candidates(compiler, &self.0, &self.1, &candidates);
        rustc_driver::Compilation::Continue
    }
}

fn print_candidates(compiler: &interface::Compiler, refactoring: &str, crate_name: &str, candidates: &[Span]) {
    let c = CandidateOutput {
        crate_name: crate_name.to_string(),
        refactoring: refactoring.to_string(),
        candidates: candidates.iter().map(|s| 
            map_span_to_index(compiler.session().source_map(), *s)
        ).map(|(file, range)| CandidatePosition {
            from: range.from.byte,
            to: range.to.byte,
            file
        }).collect()
    };
    print!("{}", serde_json::to_string(&c).unwrap());
}