use super::{arg_value, collect_extract_block_candidates, collect_box_field_candidates, CollectFieldMode};
use rustc_interface::interface;
use rustc_span::Span;
use crate::refactorings::utils::map_span_to_index;
use crate::output_types::{CandidateOutput, CandidatePosition, RefactorOutputs};

pub fn should_query_candidates(refactor_args: &[String]) -> bool {
    arg_value(refactor_args, "--query-candidates", |_| true).is_some()
}

pub fn list_candidates(refactor_args: &[String], rustc_args: &[String]) -> Result<(), i32> {

    let c = arg_value(refactor_args, "--query-candidates", |_| true).unwrap();

    let crate_name = arg_value(rustc_args, "--crate-name", |_| true).unwrap();
    let is_test = rustc_args.contains(&"--test".to_owned());

    let mut callbacks = RustcAfterParsing(c.to_string(), crate_name.to_string(), is_test);

    let emitter = Box::new(Vec::new());
    std::env::set_var("RUST_BACKTRACE", "1");
    let err = rustc_driver::run_compiler(&rustc_args, &mut callbacks, None, Some(emitter));
    err.unwrap();
    Ok(())
}


struct RustcAfterParsing(String, String, bool); // TODO: after parsing or expansion?

impl rustc_driver::Callbacks for RustcAfterParsing
{
    fn after_expansion<'tcx>(
        &mut self,
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {

        let c = crate::refactoring_invocation::AstContext::new(compiler, queries);

        let candidates = 
        match self.0.as_ref() {
            "extract-block" => collect_extract_block_candidates(queries),
            "box-field" => collect_box_field_candidates(&c, CollectFieldMode::All).unwrap(),
            "box-named-field" => collect_box_field_candidates(&c, CollectFieldMode::Named).unwrap(),
            "box-tuple-field" => collect_box_field_candidates(&c, CollectFieldMode::Tuple).unwrap(),
            _ => panic!("Unknown argument to query-candidate: `{}`", self.0)
        };

        let refactoring = match self.0.as_ref() {
            "box-named-field" |
            "box-tuple-field"  => {
                "box-field"
            },
            r => r
        };

        print_candidates(compiler, refactoring, &self.1, &candidates, self.2);
        rustc_driver::Compilation::Continue
    }
}

fn print_candidates(compiler: &interface::Compiler, refactoring: &str, crate_name: &str, candidates: &[Span], is_test: bool) {
    let c = CandidateOutput {
        crate_name: crate_name.to_string(),
        is_test,
        refactoring: refactoring.to_string(),
        candidates: candidates.iter().map(|s| 
            map_span_to_index(compiler.session().source_map(), *s)
        ).map(|(file, range)| CandidatePosition {
            from: range.from.byte,
            to: range.to.byte,
            file
        }).collect()
    };
    let outputs = RefactorOutputs {
        candidates: vec![c],
        refactorings: vec![]
    };
    print!("{}", serde_json::to_string(&outputs).unwrap());
}