use std::path::Path;
use crate::refactoring_invocation::get_sys_root;
use em_refactor_lib_types::{CandidateArgs, RefactorArgs, 
    defs::{ENV_CANDIDATE_ARGS, ENV_REFACTORING_ARGS}};

pub fn arg_value<'a>(
    args: impl IntoIterator<Item = &'a String>,
    find_arg: &str,
    pred: impl Fn(&str) -> bool,
) -> Option<&'a str> {
    let mut args = args.into_iter().map(String::as_str);

    while let Some(arg) = args.next() {
        let arg: Vec<_> = arg.splitn(2, '=').collect();
        if arg.get(0) != Some(&find_arg) {
            continue;
        }

        let value = arg.get(1).cloned().or_else(|| args.next());
        if value.as_ref().map_or(false, |p| pred(p)) {
            return value;
        }
    }
    None
}

pub fn get_candidate_args() -> Option<CandidateArgs> {
    if let Ok(args) = std::env::var(ENV_CANDIDATE_ARGS) {
        serde_json::from_str::<CandidateArgs>(&args).ok()
    } else {
        None
    }
}

pub fn get_refactor_args() -> RefactorArgs {
    std::env::var(ENV_REFACTORING_ARGS)
        .map(|s| serde_json::from_str::<RefactorArgs>(&s).unwrap())
        .unwrap()
}

///
/// Collect all arguments until '--', which should be passed to rustc
///
pub fn get_compiler_args(args: &[String]) -> Vec<String> {
    let have_sys_root = arg_value(args, "--sysroot", |_| true).is_some();
    // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
    // We're invoking the compiler programmatically, so we ignore this/
    let wrapper_mode = Path::new(&args[1]).file_stem() == Some("rustc".as_ref());

    let mut rustc_args: Vec<_>;

    if wrapper_mode {
        // we still want to be able to invoke it normally though
        rustc_args = args.iter().skip(1).map(|s| s.to_string()).collect();
    } else {
        rustc_args = args
            .iter()
            .skip(1)
            .take_while(|s| *s != "--")
            .map(|s| s.to_string())
            .collect();
        rustc_args.insert(0, "".to_owned());
    }

    // this conditional check for the --sysroot flag is there so users can call
    // `clippy_driver` directly
    // without having to pass --sysroot or anything
    if !have_sys_root {
        rustc_args.push("--sysroot".to_owned());
        rustc_args.push(get_sys_root());
    }
    rustc_args.push("--allow".to_owned());
    rustc_args.push("dead_code".to_owned());
    rustc_args.push("--allow".to_owned());
    rustc_args.push("deprecated".to_owned());
    rustc_args.push("--allow".to_owned());
    rustc_args.push("unused".to_owned());

    rustc_args
}
