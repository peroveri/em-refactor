#![feature(rustc_private)]

#[allow(unused_extern_crates)]
extern crate rustc_driver;
#[allow(unused_extern_crates)]
extern crate rustc_interface;
#[allow(unused_extern_crates)]
extern crate rustc;
#[allow(unused_extern_crates)]
extern crate syntax;

use std::process::{exit, Command};
use std::path::{Path, PathBuf};

mod my_refactor_callbacks;
mod hir_visitor;
mod ast_visitor;

fn arg_value<'a>(
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
pub fn main() {
    // eprintln!("driver.rs: args: {}", std::env::args().collect::<Vec<_>>().join(","));
    rustc_driver::init_rustc_env_logger();
    exit(
        rustc_driver::report_ices_to_stderr_if_any(move || {
            
            let mut orig_args: Vec<String> = std::env::args().collect();

            let sys_root_arg = arg_value(&orig_args, "--sysroot", |_| true);
            let have_sys_root_arg = sys_root_arg.is_some();
            let sys_root = sys_root_arg
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
                .expect("need to specify SYSROOT env var during clippy compilation, or use rustup or multirust");


            // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
            // We're invoking the compiler programmatically, so we ignore this/
            let wrapper_mode = Path::new(&orig_args[1]).file_stem() == Some("rustc".as_ref());

            if wrapper_mode {
                // we still want to be able to invoke it normally though
                orig_args.remove(1);
            }

            // this conditional check for the --sysroot flag is there so users can call
            // `clippy_driver` directly
            // without having to pass --sysroot or anything
            let mut args: Vec<String> = if have_sys_root_arg {
                orig_args.clone()
            } else {
                orig_args
                    .clone()
                    .into_iter()
                    .chain(Some("--sysroot".to_owned()))
                    .chain(Some(sys_root))
                    .collect()
            };

            // let mut default = rustc_driver::DefaultCallbacks;
            let mut my_refactor = my_refactor_callbacks::MyRefactorCallbacks {
                args: my_refactor_callbacks::def()
            };
            let callbacks: &mut (dyn rustc_driver::Callbacks + Send) = &mut my_refactor;

            // let args = std::env::args().collect::<Vec<_>>();
            rustc_driver::run_compiler(&args, callbacks, None, None)
        }).and_then(|result| result)
        .is_err() as i32
    )
}