#![feature(rustc_private)]

#[allow(unused_extern_crates)]
extern crate rustc_driver;
#[allow(unused_extern_crates)]
extern crate rustc_interface;
#[allow(unused_extern_crates)]
extern crate rustc;
#[allow(unused_extern_crates)]
extern crate syntax;
#[allow(unused_extern_crates)]
extern crate syntax_pos;

use std::process::{exit, Command};
use std::path::{Path, PathBuf};

mod refactorings;
mod refactor_args;
mod my_refactor_callbacks;
mod change;

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
    // eprintln!("MY ARGS: {}", std::env::var("MY_REFACTOR_ARGS").unwrap());
    rustc_driver::init_rustc_env_logger();
    rustc_driver::install_ice_hook();
    exit(
        rustc_driver::catch_fatal_errors(move || {
            
            let orig_args: Vec<String> = std::env::args().collect();

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

            let refactor_args: String;
            let mut rustc_args: Vec<_>;

            if wrapper_mode {
                // we still want to be able to invoke it normally though
                rustc_args = orig_args.into_iter().skip(1).collect();
                refactor_args = std::env::var("MY_REFACTOR_ARGS").unwrap();
            } else {
                let orig_args = orig_args.into_iter().skip(1).collect::<Vec<String>>();
                rustc_args = orig_args.clone().into_iter().take_while(|s| *s != "--").collect();
                let mut refactor_args_vec = orig_args.into_iter().skip_while(|s| *s != "--").skip(1).collect::<Vec<String>>();
                let file_path = rustc_args.iter().find(|s| !s.starts_with('-')).unwrap();
                refactor_args_vec.push(format!("--file={}", file_path));
                refactor_args = refactor_args_vec.join(";");
                rustc_args.insert(0, "".to_owned());
            }

            // this conditional check for the --sysroot flag is there so users can call
            // `clippy_driver` directly
            // without having to pass --sysroot or anything
            if !have_sys_root_arg {
                rustc_args.push("--sysroot".to_owned());
                rustc_args.push(sys_root);
            }
            rustc_args.push("--allow".to_owned());
            rustc_args.push("dead_code".to_owned());
            rustc_args.push("--allow".to_owned());
            rustc_args.push("deprecated".to_owned());
            rustc_args.push("--allow".to_owned());
            rustc_args.push("unused".to_owned());

            std::env::set_var("RUST_BACKTRACE", "1");
            // let mut default = rustc_driver::DefaultCallbacks;
            let my_refactor_res = my_refactor_callbacks::MyRefactorCallbacks::from_arg(refactor_args);

            if let Err(msg) = my_refactor_res {
                println!("{}", msg);
                return Ok(());
            }
            let mut my_refactor = my_refactor_res.unwrap();

            let callbacks: &mut (dyn rustc_driver::Callbacks + Send) = &mut my_refactor;

            // let args = std::env::args().collect::<Vec<_>>();
            rustc_driver::run_compiler(&rustc_args, callbacks, None, None)
        }).and_then(|result| result)
        .is_err() as i32
    )
}