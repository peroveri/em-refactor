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
    let mut args = vec!["check".to_owned(), "--quiet".to_owned()];
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
