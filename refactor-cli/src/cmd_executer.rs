use itertools::Itertools;
use refactor_lib_types::{RefactorOutputs, RefactorOutputs2};
use std::process::Command;

static DRIVER_NAME: &str = "my-refactor-driver";

// from Rerast
// Queries cargo to find the name of the current crate, then runs cargo clean to
// clean up artifacts for that package (but not dependencies). This is necessary
// in order to ensure that all the files in the current crate actually get built
// when we run cargo check. Hopefully eventually there'll be a nicer way to
// integrate with cargo such that we won't need to do this.
pub(crate) fn clean_local_targets(metadata: &Metadata, target_dir: Option<&str>) -> Result<(), i32> {
    for name in &metadata.package_names {
        let mut args = vec!["+nightly-2020-04-15".to_owned(), "clean".to_owned(), "--package".to_owned(), name.to_string()];
        if let Some(dir) = &target_dir {
            args.push(format!("--target-dir={}", dir));
        }
        // TODO: error handling
        let out = std::process::Command::new("cargo")
            .args(args)
            // .stdout(std::process::Stdio::piped())
            // .stderr(std::process::Stdio::piped())
            .output().map_err(|_| -1)?;
        if !out.status.success() {
            return Err(-2);
        }
    }
    Ok(())
}

pub(crate) struct Metadata {
    pub package_names: Vec<String>,
    pub dependency_names: Vec<String>
}
pub(crate) fn get_metadata() -> Result<Metadata, std::io::Error> {
    let mut metadata = Metadata {package_names: vec![], dependency_names: vec![]};
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
            metadata.package_names.push(name.to_string());

            if let Some(arr) = package["dependencies"].as_array() {
                for dep in arr {
                    if let Some(dep_name) = dep["name"].as_str().map(|s| s.to_string()) {
                        if !metadata.dependency_names.contains(&dep_name) {
                            metadata.dependency_names.push(dep_name.to_string());
                        }
                    }
                }
            }
        }
    }
    Ok(metadata)
}

pub(crate) fn run_refactoring_cmd(target_dir: Option<&str>, env_args: (String, String)) -> Result<RefactorOutputs2, i32> {
    
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

    let output = Command::new("cargo")
        .args(&args)
        .env("RUSTC_WRAPPER", path)
        .env(env_args.0, env_args.1)
        .stdout(std::process::Stdio::piped())
        // .stderr(std::process::Stdio::piped())
        .output().unwrap();
    
    if output.status.success() {
        let s = std::str::from_utf8(output.stdout.as_slice()).unwrap();
        
        Ok(combine_output(s))
    } else {
        let s = std::str::from_utf8(output.stderr.as_slice()).unwrap();
        eprint!("{}", s);
        Err(output.status.code().unwrap_or(-1))
    }
}

fn combine_output(s: &str) -> RefactorOutputs2 {

    let outputs = s
        .lines()
        .filter_map(|line|{
            serde_json::from_str::<RefactorOutputs>(&line).ok()
        })
        .collect::<Vec<_>>();

    let mut output2 = RefactorOutputs2::empty();
    let mut replacements = vec![];

    for o in outputs {
        output2.candidates.extend(o.candidates.into_iter().flat_map(|c| c.candidates));
        replacements.extend(o.refactorings.iter().flat_map(|c| c.replacements.clone()));
        output2.errors.extend(o.refactorings.into_iter().flat_map(|c| c.errors));    
    }
    output2.candidates = output2.candidates.into_iter().unique().sorted().collect::<Vec<_>>();
    let changes = replacements.into_iter()
        .unique()
        .sorted_by_key(|p| -(p.byte_start as i32))
        .collect::<Vec<_>>();
    if !changes.is_empty() {
        output2.changes.push(changes);    
    }
    output2.errors = output2.errors.into_iter().filter(|e| e.is_error).unique().sorted().collect::<Vec<_>>();

    output2
}