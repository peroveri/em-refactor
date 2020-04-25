use super::{arg_value, collect_extract_block_candidates, collect_box_field_all_candidates, collect_box_field_namede_candidates, collect_box_field_tuple_candidates};
use rustc_span::Span;
use crate::refactorings::utils::map_span_to_index;
use refactor_lib_types::{CandidateArgs, CandidateOutput, CandidatePosition, RefactorOutputs};
use crate::refactoring_invocation::{AstContext, is_dep, QueryResult, Query, MyRefactorCallbacks};

pub fn should_query_candidates(refactor_args: &[String]) -> bool {
    arg_value(refactor_args, "--query-candidates", |_| true).is_some()
}

fn map_to_pos_query(args: CandidateQueryArgs, f: Box<dyn Fn(&AstContext) -> QueryResult<Vec<Span>> + Send>) -> Query<RefactorOutputs> {
    Query::AfterExpansion(
        Box::new(
            move |ast| {
                let res = f(ast)?;

                let candidates = res.iter().map(|span| {
                    let (file, range) = map_span_to_index(ast.get_source_map(), *span);
                    CandidatePosition {
                        file,
                        from: range.from.byte,
                        to: range.to.byte
                    }}).collect::<Vec<_>>();

                Ok(print_candidates(args.clone(), candidates))
            }
        )
    )
}

fn map_to_query(args: CandidateQueryArgs) -> Query<RefactorOutputs> {
    match args.refactoring.as_ref() {
        "extract-block" => map_to_pos_query(args, Box::new(collect_extract_block_candidates)),
        "box-field" => map_to_pos_query(args, Box::new(collect_box_field_all_candidates)),
        "box-named-field" => map_to_pos_query(args, Box::new(collect_box_field_namede_candidates)),
        "box-tuple-field" => map_to_pos_query(args, Box::new(collect_box_field_tuple_candidates)),
        _ => panic!("Unknown argument to query-candidate: `{}`", args.refactoring)
    }
}

#[derive(Clone)]
struct CandidateQueryArgs {
    refactoring: String,
    crate_name: String,
    is_test: bool,
}

impl CandidateQueryArgs {
    fn parse(candidate: &str, rustc_args: &[String]) -> Self {
        Self {
            refactoring: candidate.to_string(),
            crate_name: arg_value(rustc_args, "--crate-name", |_| true).unwrap().to_string(),
            is_test: rustc_args.contains(&"--test".to_owned())
        }
    }
}

/// TODO: Should use the refa. invocation instead and remove this
pub fn list_candidates(candidate: &CandidateArgs, rustc_args: &[String]) -> Result<(), i32> {

    let args = CandidateQueryArgs::parse(&candidate.refactoring, rustc_args);
    let query = map_to_query(args);

    let mut callbacks = MyRefactorCallbacks::from_arg(query, is_dep(&candidate.deps, rustc_args));

    let emitter = Box::new(Vec::new());
    std::env::set_var("RUST_BACKTRACE", "1");
    let err = rustc_driver::run_compiler(&rustc_args, &mut callbacks, None, Some(emitter));
    err.unwrap();

    print!("{}", serde_json::to_string(&callbacks.result.unwrap()).unwrap());
    
    Ok(())
}

fn print_candidates(args: CandidateQueryArgs, candidates: Vec<CandidatePosition>) -> RefactorOutputs {
    let refa = match args.refactoring.as_ref() {
        "box-named-field" |
        "box-tuple-field"  => {
            "box-field".to_string()
        },
        r => r.to_string()
    };
    let c = CandidateOutput {
        crate_name: args.crate_name,
        is_test: args.is_test,
        refactoring: refa,
        candidates
    };
    let outputs = RefactorOutputs {
        candidates: vec![c],
        refactorings: vec![]
    };
    outputs
}