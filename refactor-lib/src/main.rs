#![cfg_attr(feature = "deny-warnings", deny(warnings))]
static DRIVER_NAME: &str = "my-refactor-driver";

pub fn main() {
    if let Err(code) = process(std::env::args().skip(2)) {
        std::process::exit(code);
    }
}

fn process<I>(mut old_args: I) -> Result<(), i32>
where
    I: Iterator<Item = String>,
{
    let mut args = vec!["check".to_owned()];

    for arg in old_args.by_ref() {
        if arg == "--" {
            break;
        }
        args.push(arg);
    }

    let mut path = std::env::current_exe()
        .expect("current executable path invalid")
        .with_file_name(DRIVER_NAME);
    if cfg!(windows) {
        path.set_extension("exe");
    }

    let exit_status = std::process::Command::new("cargo")
        .args(&args)
        .env("RUSTC_WRAPPER", path)
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
