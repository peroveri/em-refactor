use clap::ArgMatches;
use refactor_lib_types::{CandidateArgs, RefactorArgs, SelectionType};

pub(crate) fn get_refactor_args(m: &ArgMatches, deps: &[String]) -> RefactorArgs {
    RefactorArgs {
        file: m.value_of("file").unwrap().to_string(),
        refactoring: m.value_of("refactoring").unwrap().to_string(),
        selection: SelectionType::Range(m.value_of("selection").unwrap().to_string()),
        unsafe_: m.is_present("unsafe"),
        deps: deps.to_vec(),
        add_comment: false,
        with_changes: vec![]
    }
}
pub(crate) fn get_candidate_args(m: &ArgMatches, deps: &[String]) -> CandidateArgs {
    CandidateArgs {
        refactoring: m.value_of("refactoring").unwrap().to_string(),
        deps: deps.to_vec()
    }
}
