use super::{arg_value, collect_extract_block_candidates, collect_box_field_all_candidates, collect_box_field_namede_candidates, collect_box_field_tuple_candidates};
use rustc_span::Span;
use crate::refactorings::utils::map_span_to_index;
use crate::output_types::{CandidateOutput, CandidatePosition, RefactorOutputs};
use crate::refactoring_invocation::{AstContext, QueryResult, Query, MyRefactorCallbacks};

pub fn should_query_candidates(refactor_args: &[String]) -> bool {
    arg_value(refactor_args, "--query-candidates", |_| true).is_some()
}

fn map_to_pos_query(f: Box<dyn Fn(&AstContext) -> QueryResult<Vec<Span>> + Send>) -> Query<Vec<CandidatePosition>> {
    Query::AfterExpansion(
        Box::new(
            move |ast| {
                let res = f(ast)?;
                Ok(res.iter().map(|span| {
                    let (file, range) = map_span_to_index(ast.get_source_map(), *span);
                    CandidatePosition {
                    file,
                    from: range.from.byte,
                    to: range.to.byte
                }}).collect::<Vec<_>>())
            }
        )
    )
}

fn map_to_query(name: &str) -> Query<Vec<CandidatePosition>> {
    match name {
        "extract-block" => map_to_pos_query(Box::new(collect_extract_block_candidates)),
        "box-field" => map_to_pos_query(Box::new(collect_box_field_all_candidates)),
        "box-named-field" => map_to_pos_query(Box::new(collect_box_field_namede_candidates)),
        "box-tuple-field" => map_to_pos_query(Box::new(collect_box_field_tuple_candidates)),
        _ => panic!("Unknown argument to query-candidate: `{}`", name)
    }
}

/// TODO: Should use the refa. invocation instead and remove this
pub fn list_candidates(refactor_args: &[String], rustc_args: &[String]) -> Result<(), i32> {

    let c = arg_value(refactor_args, "--query-candidates", |_| true).unwrap();

    let crate_name = arg_value(rustc_args, "--crate-name", |_| true).unwrap();
    let is_test = rustc_args.contains(&"--test".to_owned());
    let query = map_to_query(c);

    let mut callbacks = MyRefactorCallbacks::from_arg(query);

    let emitter = Box::new(Vec::new());
    std::env::set_var("RUST_BACKTRACE", "1");
    let err = rustc_driver::run_compiler(&rustc_args, &mut callbacks, None, Some(emitter));
    err.unwrap();

    let refa = match c {
        "box-named-field" |
        "box-tuple-field"  => {
            "box-field"
        },
        r => r
    };
    print_candidates(refa, crate_name, callbacks.result.unwrap(), is_test);
    Ok(())
}

fn print_candidates(refactoring: &str, crate_name: &str, candidates: Vec<CandidatePosition>, is_test: bool) {
    let c = CandidateOutput {
        crate_name: crate_name.to_string(),
        is_test,
        refactoring: refactoring.to_string(),
        candidates
    };
    let outputs = RefactorOutputs {
        candidates: vec![c],
        refactorings: vec![]
    };
    print!("{}", serde_json::to_string(&outputs).unwrap());
}