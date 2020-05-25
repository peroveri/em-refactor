use arg_mappings::*;
use clap::{Arg, App, AppSettings, SubCommand};
use cmd_executer::*;
use invocation_error::*;
use itertools::Itertools;
use em_refactor_lib_types::{*, defs::*};

mod arg_mappings;
mod cmd_executer;
mod invocation_error;

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("Refactoring tool")
     .version("0.0.1")
     .setting(AppSettings::SubcommandRequiredElseHelp)
     .author("Per Ove Ringdal <peroveri@gmail.com>")
     .arg(Arg::with_name("target-dir")
        .long("target-dir")
        .takes_value(true))
     .arg(Arg::with_name("workspace-root")
        .long("workspace-root")
        .takes_value(true))
    .arg(Arg::with_name("single-file")
        .long("single-file")
        .help("Output the changed file instead of the diff's. Asserts that only a single file was changed."))
     .subcommand(
         SubCommand::with_name("refactor")
            .arg(Arg::with_name("refactoring")
                .help("The id of the refactoring")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("file")
                .help("File path")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("selection")
                .help("Selection on the format <byte start>:<byte end>. E.g.: 10:20")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("unsafe")
                .long("unsafe")
                .help("Skips the recompile check")))
    .subcommand(
        SubCommand::with_name("candidates")
            .arg(Arg::with_name("refactoring")
                .help("Id of the refactoring")
                .required(true)
                .takes_value(true))
            .arg(Arg::with_name("target-dir")
                .long("target-dir")
                .takes_value(true)))
}



///
/// Wrapper binary which invokes cargo check with the RUSTC_WRAPPER env var set to the binary produced by driver.rs
/// This will cause cargo to invoke the driver.rs binary with the same arguments as if the driver.rs binary was rustc.
///
/// The main.rs binary should not be linked against librustc_driver
/// The driver.rs binary will instead be linked against librustc_driver, as cargo executes this binary
/// 
/// Invoking on a single file:
/// - ``
///
///
///
pub fn main() {
    if let Err(err) = process() {
        eprint!("{}", err.message);
        std::process::exit(-1);
    }
}

fn serialize<T>(t: &T) -> InvocationResult<String> 
    where T: serde::Serialize {
    serde_json::to_string(t).map_err(|e| InvocationError::new(e.to_string()))
}

fn process() -> InvocationResult<()> {
    let matches = app().get_matches();
    let target_dir = matches.value_of("target-dir");

    if let Some(workspace_path) = matches.value_of("workspace-root") {
        std::env::set_current_dir(&workspace_path)?;
    }
    let metadata = get_metadata()?;
    let single_file = matches.is_present("single-file");

    let output = match matches.subcommand() {
        ("candidates", Some(candidate_matches)) => {
            let env_args = (ENV_CANDIDATE_ARGS.to_owned(), serialize(&get_candidate_args(candidate_matches, &metadata.dependency_names))?);
            run_crate(&metadata, target_dir, env_args)?
        },
        ("refactor", Some(refactor_matches)) => {
            
            run_refactoring(&metadata, get_refactor_args(refactor_matches, &metadata.dependency_names), target_dir)?
        },
        (subcommand, _) => panic!("Unexpected subcommand: {:?}", subcommand)
    };

    print_result(output, single_file)?;
    Ok(())
}

fn run_crate(metadata: &Metadata, target_dir: Option<&str>, env_args: (String, String)) -> InvocationResult<RefactorOutputs2> {
    // Clean local targets
    // This might cause the local cargo index to be locked, so we cannot run multiple tests on the same project in parallell.
    // might be fixed?
    // https://github.com/rust-lang/cargo/issues/7490
    //
    clean_local_targets(metadata, target_dir)?;

    run_refactoring_cmd(target_dir, env_args)
}

fn run_refactoring(metadata: &Metadata, mut refactor_args: RefactorArgs, target_dir: Option<&str>) -> InvocationResult<RefactorOutputs2> {
    match refactor_args.refactoring.as_ref() {
        defs::EXTRACT_METHOD => {
            refactor_args.add_comment = true;

            let mut combined = RefactorOutputs2::empty();
            for (refactoring, comment) in defs::extract_method_def() {

                if comment.len() > 0 {
                    refactor_args.selection = SelectionType::Comment(comment.to_string());
                }

                refactor_args.refactoring = refactoring.to_string();
                refactor_args.with_changes = combined.changes.clone();
                
                let env_args = (ENV_REFACTORING_ARGS.to_owned(), serde_json::to_string(&refactor_args).unwrap());
                let out = run_crate(metadata, target_dir, env_args)?;
                combined.changes.extend(out.changes);
                combined.errors.extend(out.errors);

                if !combined.errors.is_empty() {
                    break;
                }
            }
            Ok(combined)
        },
        _ => {

            let env_args = (ENV_REFACTORING_ARGS.to_owned(), serde_json::to_string(&refactor_args).unwrap());
            run_crate(metadata, target_dir, env_args)
        }
    }
}

fn print_result(output: RefactorOutputs2, single_file: bool) -> InvocationResult<()> {
    if single_file {
        if output.errors.is_empty() {
            print!("{}", get_file_content_with_changes(output).unwrap());
        } else {
            return Err(
                InvocationError::new(output.errors.iter().map(|e| format!("{:?}\n{}\n", e.kind, e.message)).join("\n"))
            );
        }
    } else {
        print!("{}", serde_json::to_string(&output).unwrap());
    }
    Ok(())
}


fn get_file_content_with_changes(refactor_output: RefactorOutputs2) -> Option<String> {
    use std::fs::File;
    use std::io::prelude::*;

    let mut content = String::new();
    let file_name = refactor_output.changes.first()?.first()?.file_name.to_string();
    let mut file = File::open(&file_name).unwrap();
    file.read_to_string(&mut content).unwrap();

    for changes in refactor_output.changes {
        
        for change in &changes {
            assert_eq!(file_name, change.file_name);
            let s1 = &content[..(change.byte_start) as usize];
            let s2 = &content[(change.byte_end) as usize..];
            content = format!("{}{}{}", s1, change.replacement, s2);
        }
    }
    
    Some(content)
}
