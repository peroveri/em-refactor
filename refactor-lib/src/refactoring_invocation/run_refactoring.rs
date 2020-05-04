use refactor_lib_types::{FileStringReplacement, RefactorArgs};
use crate::refactoring_invocation::{arg_value, argument_list_to_refactor_def, AstDiff, from_error, from_success, MyRefactorCallbacks, QueryResult, RefactoringErrorInternal, rustc_rerun, serialize, InMemoryFileLoader};
use itertools::Itertools;

pub fn run_refactoring_and_output_result(refactor_args: &RefactorArgs, rustc_args: Vec<String>) -> Result<(), i32> {
    
    let output = match run_refactoring(refactor_args, &rustc_args) {
        Err(err) => from_error(&rustc_args, err, &refactor_args.refactoring),
        Ok(astdiff) => from_success(&rustc_args, astdiff.0)
    };
    print!("{}", serialize(&output).unwrap());
    Ok(())
}

fn run_refactoring(refactor_args: &RefactorArgs, rustc_args: &Vec<String>) -> QueryResult<AstDiff> {


    // 1. Run refactoring callbacks
    let refactor_res = run_refactoring_internal(rustc_args, refactor_args)?;

    // 2. Rerun the compiler to check if any errors were introduced
    // Runs with default callbacks
    if !refactor_args.unsafe_ && !refactor_res.0.is_empty() {
        let mut combined = refactor_args.with_changes.clone();
        combined.push(refactor_res.0.clone());
        rustc_rerun(combined, &rustc_args)?;
    }

    Ok(refactor_res)
}

fn run_refactoring_internal(rustc_args: &[String], refactor_args: &RefactorArgs) -> QueryResult<AstDiff> {
    
    let refactor_def = argument_list_to_refactor_def(refactor_args.clone())?;

    let mut my_refactor = MyRefactorCallbacks::from_arg(refactor_def, is_dep(&refactor_args.deps, rustc_args));

    let callbacks: &mut (dyn rustc_driver::Callbacks + Send) = &mut my_refactor;

    std::env::set_var("RUST_BACKTRACE", "1");

    let mut file_loader = Box::new(InMemoryFileLoader::new(
        rustc_span::source_map::RealFileLoader,
    ));
    file_loader.add_changes(refactor_args.with_changes.clone());

    let emitter = Box::new(Vec::new());
    // TODO: looks like the errors are not caught here?
    // Should set own errors on the Callbacks struct
    let err = rustc_driver::run_compiler(&rustc_args, callbacks, Some(file_loader), Some(emitter));
    // let err = rustc_driver::catch_fatal_errors(|| {
    //     rustc_driver::run_compiler(&rustc_args, callbacks, None, Some(emitter))
    // });
    if err.is_err() {
        return Err(RefactoringErrorInternal::compile_err());
    }

    check_no_overlapping_changes(&my_refactor.result)?;

    my_refactor.result
}

fn check_no_overlapping_changes(res: &QueryResult<AstDiff>) -> QueryResult<()> {

    if let Ok(diff) = res {
        for (_, group) in &diff.0.iter()
            .sorted_by_key(|e| e.file_name.to_string())
            .group_by(|e| e.file_name.to_string()) {
            
            let items = group.collect::<Vec<_>>();

            for i in 0..items.len() - 1 {
                for j in i + 1..items.len() - 1 {
                    
                    let (itemi, itemj) = (items[i], items[j]);
                    let range_i = itemi.byte_start..itemi.byte_end;
                    if range_i.contains(&itemj.byte_start) || range_i.contains(&itemj.byte_end) {
                        return Err(RefactoringErrorInternal::int("overlapping range"));
                    }
                }   
            }
        }

    }

    Ok(())
}


pub fn get_file_content(changes: &[FileStringReplacement]) -> Option<String> {
    use std::fs::File;
    use std::io::prelude::*;
    let mut changes = changes.to_vec();
    changes.sort_by_key(|c| c.byte_start);
    changes.reverse();

    let mut file = File::open(&changes[0].file_name).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    
    for change in &changes {
        let s1 = &content[..(change.byte_start) as usize];
        let s2 = &content[(change.byte_end) as usize..];
        content = format!("{}{}{}", s1, change.replacement, s2);
    }

    return Some(content);
}

pub fn is_dep(deps: &[String], rustc_arg: &[String]) -> bool {
    (if let Some(val) = arg_value(rustc_arg, "--crate-name", |_| true) {
        deps.iter().any(|s| s == val)
    } else {
        false
    }) || // libraries can be dependencies of examples
    if let Some(val) = arg_value(rustc_arg, "--crate-type", |_| true) {
        val == "lib"
    } else {
        false
    }
}
