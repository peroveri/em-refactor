#![feature(rustc_private)]

// FIXME: switch to something more ergonomic here, once available.
// (Currently there is no way to opt into sysroot crates without `extern crate`.)
// #[allow(unused_extern_crates)]
extern crate rustc_driver;
// #[allow(unused_extern_crates)]
extern crate rustc;
extern crate rustc_errors;
extern crate rustc_interface;
extern crate syntax;

use refactor_lib::RefactorArgs;

use std::path::PathBuf;
use std::process::{exit, Command};

mod arg_value;
mod rc_callbacks;

fn get_sys_root() -> String {
    let sys_root_arg: Option<&str> = Option::None;
    sys_root_arg
        .map(PathBuf::from)
        .or_else(|| std::env::var("SYSROOT").ok().map(PathBuf::from))
        .or_else(|| {
            let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
            let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));
            home.and_then(|home| {
                toolchain.map(|toolchain| {
                    let mut path = PathBuf::from(home);
                    path.push("toolchains");
                    path.push(toolchain);
                    path
                })
            })
        })
        .or_else(|| {
            Command::new("rustc")
                .arg("--print")
                .arg("sysroot")
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .map(|s| PathBuf::from(s.trim()))
        })
        .or_else(|| option_env!("SYSROOT").map(PathBuf::from))
        .map(|pb| pb.to_string_lossy().to_string())
        .expect(
            "need to specify SYSROOT env var during clippy compilation, or use rustup or multirust",
        )
}

fn get_arg(args: &Vec<String>, to_find: &str) -> Option<String> {
    let mut it = args.iter();
    while let Some(e) = it.next() {
        if e == &*to_find {
            if let Some(v) = it.next() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn parse_args(args: &Vec<String>) -> Option<RefactorArgs> {
    if let Some(method) = get_arg(&args, "--method") {
        if let Some(selection) = get_arg(&args, "--selection") {
            if let Some(file) = get_arg(&args, "--file") {
                return Some(RefactorArgs {
                    method,
                    selection,
                    file
                });
            }
        }
    }
    None
}

pub fn main() {
    rustc_driver::init_rustc_env_logger();
    exit(
        rustc_driver::report_ices_to_stderr_if_any(move || {
            let mut args: Vec<_> = std::env::args().collect();
            args.resize(1, "".to_string());
            args.extend(vec!["--sysroot".to_owned(), get_sys_root(), "src/main.rs".to_owned()]);
            let mut clippy = rc_callbacks::ClippyCallbacks {
                args: parse_args(&std::env::args().collect()).expect("args not provided")
            };
            let callbacks: &mut (dyn rustc_driver::Callbacks + Send) = &mut clippy;
            rustc_driver::run_compiler(&args, callbacks, None, None)
        })
        .and_then(|result| result)
        .is_err() as i32,
    )
}
