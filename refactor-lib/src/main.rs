static DRIVER_NAME: &str = "my-refactor-driver";

const MY_REFACTOR_HELP: &str = r#"Refactorings for the Rust programming language.

Usage: 
    cargo my-refactor [<refactor-opts>...] [--] [cargo-check-options]

[cargo-check-options] are passed to cargo check.

<refactor-opts>:
[--help]
[--version]
--refactoring=[]
--selection=[]
--file=[]
[--workspace-root=<path>]
[--unsafe]
[--output-changes-as-json]
"#;

///
/// Wrapper binary which invokes cargo check with the RUSTC_WRAPPER env var set to the binary produced by driver.rs
/// This will cause cargo to invoke the driver.rs binary with the same arguments as if the driver.rs binary was rustc.
///
/// Invoking on a single file:
/// - ``
///
///
///
pub fn main() {
    if let Err(code) = process(std::env::args().skip(1)) {
        std::process::exit(code);
    }
}

fn get_option(args: &[String], option: &str) -> Option<String> {
    for arg in args {
        if arg.starts_with(option) {
            return Some(arg[option.len()..].to_string());
        }
    }
    None
}

fn process<I>(mut old_args: I) -> Result<(), i32>
where
    I: Iterator<Item = String>,
{
    let mut my_refactor_args = old_args
        .by_ref()
        .take_while(|s| s != "--")
        .collect::<Vec<_>>();
    let mut args = vec!["+nightly".to_owned(), "check".to_owned(), "--quiet".to_owned(), "--tests".to_owned(), "--benches".to_owned(), "--examples".to_owned(), "--bins".to_owned()];
    args.extend(old_args.collect::<Vec<_>>());

    // TODO: collect as JSON
    if my_refactor_args.contains(&"--help".to_owned()) {
        println!("{}", MY_REFACTOR_HELP);
        return Ok(());
    }

    if my_refactor_args.contains(&"--version".to_owned()) {
        println!("Version: 0.0.1");
        return Ok(());
    }

    let mut path = std::env::current_exe()
        .expect("current executable path invalid")
        .with_file_name(DRIVER_NAME);
    if cfg!(windows) {
        path.set_extension("exe");
    }

    if let Some(workspace_path) = get_option(&my_refactor_args, "--workspace-root=") {
        let res = std::env::set_current_dir(&workspace_path);
        if res.is_err() {
            eprintln!(
                "Couldn't set current directory to: {}. Current dir is: {:?}",
                workspace_path,
                std::env::current_dir()
            );
            return Err(1);
        }
        my_refactor_args.retain(|s| !s.starts_with("--workspace-root"));
    }

    // Clean local targets
    // This might cause the local cargo index to be locked, so we cannot run multiple tests on the same project in parallell.
    // might be fixed?
    // https://github.com/rust-lang/cargo/issues/7490
    //
    clean_local_targets(get_option(&args, "--target-dir=")).unwrap();

    let exit_status = std::process::Command::new("cargo")
        .args(&args)
        .env("RUSTC_WRAPPER", path)
        .env("MY_REFACTOR_ARGS", my_refactor_args.join(";"))
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");
        
    if exit_status.success() {
        Ok(())
    } else {
        Err(exit_status.code().unwrap_or(-1))
    }
}

// from Rerast
// Queries cargo to find the name of the current crate, then runs cargo clean to
// clean up artifacts for that package (but not dependencies). This is necessary
// in order to ensure that all the files in the current crate actually get built
// when we run cargo check. Hopefully eventually there'll be a nicer way to
// integrate with cargo such that we won't need to do this.
fn clean_local_targets(target_dir: Option<String>) -> Result<(), failure::Error> {
    let output = std::process::Command::new("cargo")
        .args(vec!["metadata", "--no-deps", "--format-version=1"])
        .stdout(std::process::Stdio::piped())
        .output()?;
    assert!(
        output.status.success(),
        "cargo metadata failed:\n{}",
        std::str::from_utf8(output.stderr.as_slice())?
    );
    let metadata_str = std::str::from_utf8(output.stdout.as_slice())?;
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

#[test]
fn test_get_option_some() {
    assert_eq!(
        get_option(
            &["--workspace-root=./some-path".to_owned()],
            "--workspace-root="
        ),
        Some("./some-path".to_owned())
    );
}
#[test]
fn test_get_option_none() {
    assert_eq!(get_option(&["...".to_owned()], "--workspace-root"), None);
}
