use refactor_lib_types::*;
use clap::{Arg, App, AppSettings, ArgMatches, SubCommand};
use std::process::Command;
use itertools::Itertools;

static DRIVER_NAME: &str = "my-refactor-driver";

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
                .help("Selection on format 10:20")
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
    if let Err(code) = process() {
        std::process::exit(code);
    }
}

fn get_refactor_args(m: &ArgMatches, deps: Vec<String>) -> RefactorArgs {
    RefactorArgs {
        file: m.value_of("file").unwrap().to_string(),
        refactoring: m.value_of("refactoring").unwrap().to_string(),
        selection: SelectionType::Range(m.value_of("selection").unwrap().to_string()),
        unsafe_: m.is_present("unsafe"),
        deps
    }
}
fn get_candidate_args(m: &ArgMatches, deps: Vec<String>) -> CandidateArgs {
    CandidateArgs {
        refactoring: m.value_of("refactoring").unwrap().to_string(),
        deps
    }
}

fn process() -> Result<(), i32> {
    let matches = app().get_matches();
    let target_dir = matches.value_of("target-dir");

    if let Some(workspace_path) = matches.value_of("workspace-root") {
        let res = std::env::set_current_dir(&workspace_path);
        if res.is_err() {
            eprintln!(
                "Couldn't set current directory to: {}. Current dir is: {:?}",
                workspace_path,
                std::env::current_dir()
            );
            return Err(1);
        }
    }

    run_crate(&matches, target_dir)
}

fn serialize_args(m: &ArgMatches, deps: Vec<String>) -> (String, String) {
    if let Some(subcommand_matches) = m.subcommand_matches("candidates") {
        ("CANDIDATE_ARGS".to_owned(), serde_json::to_string(&get_candidate_args(subcommand_matches, deps)).unwrap())
    } else if let Some(subcommand_matches) = m.subcommand_matches("refactor") {
        ("REFACTORING_ARGS".to_owned(), serde_json::to_string(&get_refactor_args(subcommand_matches, deps)).unwrap())
    } else {
        panic!("Unexpected subcommand: {:?}", m.subcommand_name());
    }
}

fn run_crate(matches: &ArgMatches, target_dir: Option<&str>) -> Result<(), i32> {
    let mut path = std::env::current_exe()
        .expect("current executable path invalid")
        .with_file_name(DRIVER_NAME);
    if cfg!(windows) {
        path.set_extension("exe");
    }
    let mut args = vec!["+nightly-2020-04-15".to_owned(), "check".to_owned(), "-j".to_owned(), "1".to_owned(), "--quiet".to_owned(), "--all-targets".to_owned()];

    if let Some(arg) = target_dir {
        args.push(format!("--target-dir={}", arg));
    }

    // Clean local targets
    // This might cause the local cargo index to be locked, so we cannot run multiple tests on the same project in parallell.
    // might be fixed?
    // https://github.com/rust-lang/cargo/issues/7490
    //
    let deps = clean_local_targets(target_dir).unwrap();
    let env_args = serialize_args(matches, deps);

    let output = Command::new("cargo")
        .args(&args)
        .env("RUSTC_WRAPPER", path)
        .env(env_args.0, env_args.1)
        .stdout(std::process::Stdio::piped())
        .output().unwrap();
    
    if output.status.success() {
        let s = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        let refactor_output = combine_output(s);

        if matches.is_present("single-file") {
            if refactor_output.errors.is_empty() {
                print!("{}", get_file_content_with_changes(refactor_output));
            } else {
                eprint!("{}", refactor_output.errors.iter().map(|e| format!("{:?}\n{}\n", e.kind, e.message)).join("\n"));
                return Err(-1);
            }
        } else {
            print!("{}", serde_json::to_string(&refactor_output).unwrap());
            eprint!("{}", std::str::from_utf8(output.stderr.as_slice()).unwrap());
        }
        Ok(())
    } else {
        let s = std::str::from_utf8(output.stderr.as_slice()).unwrap();
        eprint!("{}", s);
        Err(output.status.code().unwrap_or(-1))
    }
}

// from Rerast
// Queries cargo to find the name of the current crate, then runs cargo clean to
// clean up artifacts for that package (but not dependencies). This is necessary
// in order to ensure that all the files in the current crate actually get built
// when we run cargo check. Hopefully eventually there'll be a nicer way to
// integrate with cargo such that we won't need to do this.
fn clean_local_targets(target_dir: Option<&str>) -> Result<Vec<String>, std::io::Error> {
    let mut deps = vec![];
    let output = std::process::Command::new("cargo")
        .args(vec!["metadata", "--no-deps", "--format-version=1"])
        .stdout(std::process::Stdio::piped())
        .output()?;
    assert!(
        output.status.success(),
        "cargo metadata failed:\n{}",
        std::str::from_utf8(output.stderr.as_slice()).unwrap()
    );
    let metadata_str = std::str::from_utf8(output.stdout.as_slice()).unwrap();
    let parsed: serde_json::Value = match serde_json::from_str(metadata_str) {
        Ok(v) => v,
        Err(e) => panic!("Error parsing metadata JSON: {:?}", e),
    };
    for package in parsed["packages"].as_array().unwrap() {
        if let Some(name) = package["name"].as_str() {
            // // TODO: Remove once #10 is fixed.
            // if std::env::var("RERAST_FULL_CARGO_CLEAN") == Ok("1".to_string()) {
            //     std::process::Command::new("cargo")
            //         .args(vec!["clean"])
            //         .status()?;
            // } else {
            let mut args = vec!["clean".to_owned(), "--package".to_owned(), name.to_string()];
            if let Some(dir) = &target_dir {
                args.push(format!("--target-dir={}", dir));
            }
            std::process::Command::new("cargo").args(args).status()?;

            if let Some(arr) = package["dependencies"].as_array() {
                for dep in arr {
                    if let Some(dep_name) = dep["name"].as_str().map(|s| s.to_string()) {
                        if !deps.contains(&dep_name) {
                            deps.push(dep_name);
                        }
                    }
                }
            }

            // }
        }
    }
    Ok(deps)
}

fn combine_output(s: &str) -> RefactorOutputs2 {

    let outputs = s
        .lines()
        .filter_map(|line|{
            serde_json::from_str::<RefactorOutputs>(&line).ok()
        })
        .collect::<Vec<_>>();

    let mut output2 = RefactorOutputs2::empty();

    for o in outputs {
        output2.candidates.extend(o.candidates.into_iter().flat_map(|c| c.candidates));    
        output2.changes.extend(o.refactorings.iter().flat_map(|c| c.replacements.clone()));    
        output2.errors.extend(o.refactorings.into_iter().flat_map(|c| c.errors));    
    }
    output2.candidates = output2.candidates.into_iter().unique().sorted().collect::<Vec<_>>();
    output2.changes = output2.changes.into_iter().unique().sorted().collect::<Vec<_>>();
    output2.errors = output2.errors.into_iter().filter(|e| e.is_error).unique().sorted().collect::<Vec<_>>();

    output2
}

fn get_file_content_with_changes(refactor_output: RefactorOutputs2) -> String {
    get_file_content(&refactor_output.changes).unwrap()
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