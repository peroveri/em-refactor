use refactor_lib_types::*;
use clap::{Arg, App, ArgMatches};

static DRIVER_NAME: &str = "my-refactor-driver";

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("Refactoring tool")
     .version("0.0.1")
     .author("Per Ove Ringdal <peroveri@gmail.com>")
     .arg(Arg::with_name("refactoring")
          .short("r")
          .long("refactoring")
          .value_name("refactoring")
          .help("Id of the refactoring")
          .takes_value(true))
    .arg(Arg::with_name("query-candidates")
          .long("query-candidates")
          .value_name("REFACTORING")
          .help("Id of the refactoring")
          .takes_value(true))
     .arg(Arg::with_name("selection")
          .short("s")
          .long("selection")
          .help("Selection on format from:to")
          .takes_value(true))
    .arg(Arg::with_name("file")
        .short("f")
        .long("file")
        .help("File name")
        .takes_value(true))
    .arg(Arg::with_name("workspace-root")
        .long("workspace-root")
        .takes_value(true))
    .arg(Arg::with_name("unsafe")
        .long("unsafe"))
    .arg(Arg::with_name("output-replacements-as-json")
        .long("output-replacements-as-json"))
    .arg(Arg::with_name("single-file")
        .long("single-file"))
    .arg(Arg::with_name("target-dir")
        .long("target-dir")
        .takes_value(true))
    .arg(Arg::with_name("cargo-args")
        .raw(true))
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

fn get_args(m: &ArgMatches) -> RefactorArgs {
    RefactorArgs {
        file: m.value_of("file").map(|s| s.to_string()),
        output_replacements_as_json: m.is_present("output-replacements-as-json"),
        query_candidates: m.value_of("query-candidates").map(|s| s.to_string()),
        refactoring: m.value_of("refactoring").map(|s| s.to_string()),
        selection: m.value_of("selection").map(|s| s.to_string()),
        single_file: m.is_present("single-file"),
        usafe: m.is_present("usafe"),
    }
}

fn process() -> Result<(), i32> {
    let matches = app().get_matches();
    let cargo_args = matches.value_of("cargo-args");
    let refactor_args = get_args(&matches);
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

    if refactor_args.single_file {
        run_single(refactor_args, target_dir)
    } else {
        run_crate(refactor_args, target_dir, cargo_args)
    }
}

fn run_single(refactor_args: RefactorArgs, target_dir: Option<&str>) -> Result<(), i32> {
    let mut path = std::env::current_exe()
        .expect("current executable path invalid")
        .with_file_name(DRIVER_NAME);
    if cfg!(windows) {
        path.set_extension("exe");
    }
    let output = std::process::Command::new(path)
        .arg(refactor_args.file.clone().unwrap())
        .arg(format!("--out-dir={}", target_dir.unwrap()))
        .env("MY_REFACTOR_ARGS", serde_json::to_string(&refactor_args).unwrap())
        .output().unwrap();

    if output.status.success() {
        let s = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        print!("{}", combine_output(s));
        eprint!("{}", combine_output(std::str::from_utf8(output.stdout.as_slice()).unwrap()));
        Ok(())
    } else {
        let s = std::str::from_utf8(output.stderr.as_slice()).unwrap();
        eprint!("{}", combine_output(s));
        Err(output.status.code().unwrap_or(-1))
    }
}

fn run_crate(refactor_args: RefactorArgs, target_dir: Option<&str>, cargo_args: Option<&str>) -> Result<(), i32> {
    
    let mut path = std::env::current_exe()
        .expect("current executable path invalid")
        .with_file_name(DRIVER_NAME);
    if cfg!(windows) {
        path.set_extension("exe");
    }
    let mut args = vec!["+nightly-2020-04-15".to_owned(), "check".to_owned(), "-j".to_owned(), "1".to_owned(), "--quiet".to_owned(), "--tests".to_owned(), "--benches".to_owned(), "--examples".to_owned(), "--bins".to_owned()];

    if let Some(arg) = target_dir {
        args.push(format!("--target-dir={}", arg));
    }
    if let Some(arg) = cargo_args {
        args.push(arg.to_string());
    }

    // Clean local targets
    // This might cause the local cargo index to be locked, so we cannot run multiple tests on the same project in parallell.
    // might be fixed?
    // https://github.com/rust-lang/cargo/issues/7490
    //
    clean_local_targets(target_dir).unwrap();

    let output = std::process::Command::new("cargo")
        .args(&args)
        .env("RUSTC_WRAPPER", path)
        .env("MY_REFACTOR_ARGS", serde_json::to_string(&refactor_args).unwrap())
        .stdout(std::process::Stdio::piped())
        .output().unwrap();
    
    if output.status.success() {
        let s = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        print!("{}", combine_output(s));
        eprint!("{}", std::str::from_utf8(output.stderr.as_slice()).unwrap());
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
fn clean_local_targets(target_dir: Option<&str>) -> Result<(), std::io::Error> {
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
            // }
        }
    }
    Ok(())
}

fn combine_output(s: &str) -> String {
    if s.starts_with(r#"{"candidates":"#) {
        let mut outputs = RefactorOutputs{candidates: vec![], refactorings: vec![]};
        for line in s.split("\n") {
            if line.trim().len() > 0 {
                outputs.extend(serde_json::from_str::<RefactorOutputs>(&line).unwrap());
            }
        }
        outputs.sort();
        format!("{}", serde_json::to_string(&outputs).unwrap())
    } else {
        s.to_string()
    }
}
