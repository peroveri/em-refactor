use super::{arg_value, collect_extract_block_candidates, collect_box_field_all_candidates, collect_box_field_namede_candidates, collect_box_field_tuple_candidates};
use rustc_span::Span;
use crate::refactorings::utils::map_span_to_index;
use refactor_lib_types::{CandidateArgs, CandidateOutput, CandidatePosition, RefactorErrorType, RefactoringError, RefactorOutputs, defs::{BOX_FIELD_CANDIDATES, EXTRACT_METHOD_CANDIDATES}};
use crate::refactoring_invocation::{AstContext, is_dep, QueryResult, Query, RefactoringErrorInternal, MyRefactorCallbacks, from_error, serialize};

pub fn should_query_candidates(refactor_args: &[String]) -> bool {
    arg_value(refactor_args, "--query-candidates", |_| true).is_some()
}

fn map_to_pos_query(args: CandidateQueryArgs, f: Box<dyn Fn(&AstContext) -> QueryResult<Vec<Span>> + Send>) -> Query<CandidateOutput> {
    Query::AfterExpansion(
        Box::new(
            move |ast| {
                let res = f(ast)?;

                let mut candidates = vec![];
                for span in res.into_iter() {
                    let (file, range) = map_span_to_index(ast.get_source_map(), span)?;
                    candidates.push(
                        CandidatePosition {
                            file,
                            from: range.from.byte,
                            to: range.to.byte
                        }
                    );
                }

                Ok(map_candidates_to_output(args.clone(), candidates))
            }
        )
    )
}

fn map_to_query(args: CandidateQueryArgs) -> QueryResult<Query<CandidateOutput>> {
    match args.refactoring.as_ref() {
        EXTRACT_METHOD_CANDIDATES => Ok(map_to_pos_query(args, Box::new(collect_extract_block_candidates))),
        BOX_FIELD_CANDIDATES => Ok(map_to_pos_query(args, Box::new(collect_box_field_all_candidates))),
        "box-named-field" => Ok(map_to_pos_query(args, Box::new(collect_box_field_namede_candidates))),
        "box-tuple-field" => Ok(map_to_pos_query(args, Box::new(collect_box_field_tuple_candidates))),
        _ => Err(RefactoringErrorInternal::invalid_argument(format!("Unknown argument to query-candidate: `{}`", args.refactoring)))
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

pub fn list_candidates_and_print_result(candidate: &CandidateArgs, rustc_args: &[String]) {
    let output = 
        list_candidates(candidate, &rustc_args)
        .unwrap_or_else(|x| from_error(&rustc_args, x));
    print!("{}", serialize(&output).unwrap());
}

/// TODO: Should use the refa. invocation instead and remove this
fn list_candidates(candidate: &CandidateArgs, rustc_args: &[String]) -> QueryResult<RefactorOutputs> {

    let args = CandidateQueryArgs::parse(&candidate.refactoring, rustc_args);
    let query = map_to_query(args.clone())?;

    let mut callbacks = MyRefactorCallbacks::from_arg(query, is_dep(&candidate.deps, rustc_args));

    std::env::set_var("RUST_BACKTRACE", "1");
    rustc_driver::run_compiler(&rustc_args, &mut callbacks, None, None).map_err(|_| RefactoringErrorInternal::compile_err()).unwrap();

    let result = match callbacks.result {
        Ok(r) => RefactorOutputs::from_candidate(r),
        Err(err) => RefactorOutputs::from_candidate(map_err_to_output(args, err))
    };
    Ok(result)
}

fn map_candidates_to_output(args: CandidateQueryArgs, candidates: Vec<CandidatePosition>) -> CandidateOutput {
    let refa = match args.refactoring.as_ref() {
        "box-named-field" |
        "box-tuple-field"  => {
            BOX_FIELD_CANDIDATES.to_string()
        },
        r => r.to_string()
    };
    CandidateOutput {
        crate_name: args.crate_name,
        is_test: args.is_test,
        refactoring: refa,
        candidates,
        errors: vec![]
    }
}
fn map_err_to_output(args: CandidateQueryArgs, err: RefactoringErrorInternal) -> CandidateOutput {
    let refa = match args.refactoring.as_ref() {
        "box-named-field" |
        "box-tuple-field"  => {
            BOX_FIELD_CANDIDATES.to_string()
        },
        r => r.to_string()
    };
    CandidateOutput {
        crate_name: args.crate_name,
        is_test: args.is_test,
        refactoring: refa,
        candidates: vec![],
        errors: vec![RefactoringError {
            is_error: true,
            message: err.message,
            kind: RefactorErrorType::Internal,
            codes: vec![]
        }]
    }
}