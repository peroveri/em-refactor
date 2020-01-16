#![feature(rustc_private)]

// Need to add compiler dependencies, as they are not listed in Cargo.toml
extern crate rustc;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_span;
extern crate rustc_typeck;
extern crate syntax;

use std::path::{Path, PathBuf};
use std::process::{exit, Command};

mod change;
mod change_serialize;
mod extra;
mod file_loader;
mod my_refactor_callbacks;
mod refactor_definition;
mod refactor_definition_parser;
mod refactorings;

#[cfg(test)]
mod test_utils;
#[cfg(test)]
use test_utils::{create_test_span, run_after_analysis/*, run_after_expansion, run_after_parsing*/};

enum RefactorStatusCodes {
    Success = 0,
    InputDoesNotCompile = 1,
    RefactoringProcucedBrokenCode = 2,
    BadFormatOnInput = 3,
    // Serializing = 4,
    RustcPassFailed = 5,
    InternalRefactoringError = 6,
}

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

// Call compiler with refactoring tools callbacks
// Args to the compiler: file, sysroot, ++
// args to the refactoring tools: refactoringargs
// returns: a set of changes
//
fn is_wrapper_mode(args: &[String]) -> bool {
    Path::new(&args[1]).file_stem() == Some("rustc".as_ref())
}
fn get_file_path(args: &[String]) -> Option<&String> {
    args.iter().find(|s| !s.starts_with('-'))
}
fn get_refactor_args(args: &[String]) -> Vec<String> {
    if is_wrapper_mode(&args) {
        std::env::var("MY_REFACTOR_ARGS")
            .unwrap()
            .split(';')
            .map(|s| s.to_string())
            .collect()
    } else {
        let mut ret = args
            .iter()
            .skip_while(|s| *s != "--")
            .skip(1)
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        ret.push(format!("--file={}", get_file_path(args).unwrap()));
        ret
    }
}

fn get_sys_root() -> String {
    std::env::var("SYSROOT")
        .ok()
        .map(PathBuf::from)
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

///
/// Collect all arguments until '--', which should be passed to rustc
///
fn get_compiler_args(args: &[String]) -> Vec<String> {
    let have_sys_root = arg_value(args, "--sysroot", |_| true).is_some();
    // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
    // We're invoking the compiler programmatically, so we ignore this/
    let wrapper_mode = Path::new(&args[1]).file_stem() == Some("rustc".as_ref());

    let mut rustc_args: Vec<_>;

    if wrapper_mode {
        // we still want to be able to invoke it normally though
        rustc_args = args.iter().skip(1).map(|s| s.to_string()).collect();
    } else {
        rustc_args = args
            .iter()
            .skip(1)
            .take_while(|s| *s != "--")
            .map(|s| s.to_string())
            .collect();
        rustc_args.insert(0, "".to_owned());
    }

    // this conditional check for the --sysroot flag is there so users can call
    // `clippy_driver` directly
    // without having to pass --sysroot or anything
    if !have_sys_root {
        rustc_args.push("--sysroot".to_owned());
        rustc_args.push(get_sys_root());
    }
    rustc_args.push("--allow".to_owned());
    rustc_args.push("dead_code".to_owned());
    rustc_args.push("--allow".to_owned());
    rustc_args.push("deprecated".to_owned());
    rustc_args.push("--allow".to_owned());
    rustc_args.push("unused".to_owned());

    rustc_args
}

/// Using Rerast's solution
/// https://github.com/google/rerast/blob/46dacd520f6bc63f4c37d9593b1b5163fc81611c/src/lib.rs
fn is_compiling_dependency(args: &[String]) -> bool {
    if let Some(path) = args.iter().find(|arg| arg.ends_with(".rs")) {
        Path::new(path).is_absolute()
    } else {
        false
    }
}

///
/// 1. Run rustc with refactoring callbacks
/// 2. Run rustc with no callbacks, but with changes applied by the refactorings
///
fn run_rustc() -> Result<(), i32> {
    // get compiler and refactoring args from input and environment
    let std_env_args = std::env::args().collect::<Vec<_>>();
    let rustc_args = get_compiler_args(&std_env_args);
    if rustc_args.contains(&"--print=cfg".to_owned()) || is_compiling_dependency(&rustc_args) {
        let mut default = rustc_driver::DefaultCallbacks;
        let err = rustc_driver::run_compiler(&rustc_args, &mut default, None, None); /* Some(Box::new(Vec::new())) */
        return if err.is_err() {
            Err(RefactorStatusCodes::RustcPassFailed as i32)
        } else {
            Ok(())
        };
    }

    let refactor_args = get_refactor_args(&std_env_args);

    if refactor_args.contains(&"--provide-type".to_owned()) {
        let selection = arg_value(&refactor_args, "--selection", |_| true).unwrap();
        let file_name = arg_value(&refactor_args, "--file", |_| true).unwrap();
        if let Ok(()) = extra::provide_type(&rustc_args, file_name, selection) {
            return Ok(());
        } else {
            return Err(RefactorStatusCodes::RustcPassFailed as i32);
        }
    }

    let refactor_def = refactor_definition_parser::argument_list_to_refactor_def(&refactor_args);
    if let Err(err) = refactor_def {
        eprintln!("{}", err);
        return Err(RefactorStatusCodes::BadFormatOnInput as i32);
    }
    let refactor_def = refactor_def.unwrap();
    let mut my_refactor = my_refactor_callbacks::MyRefactorCallbacks::from_arg(refactor_def);

    // 1. Run refactoring callbacks
    let callbacks: &mut (dyn rustc_driver::Callbacks + Send) = &mut my_refactor;

    std::env::set_var("RUST_BACKTRACE", "1");

    let emitter = Box::new(Vec::new());
    // TODO: looks like the errors are not caught here?
    // Should set own errors on the Callbacks struct
    let err = rustc_driver::run_compiler(&rustc_args, callbacks, None, Some(emitter));
    // let err = rustc_driver::catch_fatal_errors(|| {
    //     rustc_driver::run_compiler(&rustc_args, callbacks, None, Some(emitter))
    // });
    if err.is_err() {
        if let Some(msg) = my_refactor.content {
            eprintln!("{}", msg);
        } else {
            eprintln!("failed during refactoring");
        }
        return Err(RefactorStatusCodes::InputDoesNotCompile as i32);
    }

    // 2. Rerun the compiler to check if any errors were introduced
    // Runs with default callbacks
    let changes = my_refactor.result.clone().ok().unwrap_or_else(|| vec![]);
    let content = my_refactor.content.clone().unwrap_or_else(|| "".to_owned());
    let replacements = my_refactor.file_replace_content.clone();

    if let Err(err) = my_refactor.result {
        if err.code == crate::refactor_definition::InternalErrorCodes::FileNotFound &&
            refactor_args.contains(&"--ignore-missing-file".to_owned()) {
                return Ok(());
            }
        eprintln!("{}", err.message);
        return Err(RefactorStatusCodes::InternalRefactoringError as i32);
    }

    if !refactor_args.contains(&"--unsafe".to_owned()) {
        let mut default = rustc_driver::DefaultCallbacks;

        let mut file_loader = Box::new(file_loader::InMemoryFileLoader::new(
            rustc_span::source_map::RealFileLoader,
        ));
        file_loader.add_changes(my_refactor.result.clone().unwrap());

        let emitter = Box::new(Vec::new());
        let err =
            rustc_driver::run_compiler(&rustc_args, &mut default, Some(file_loader), Some(emitter));
        // let err = rustc_driver::catch_fatal_errors(|| {
        //     let err = rustc_driver::run_compiler(&rustc_args, &mut default, Some(file_loader), Some(emitter));
        //     if let Err(err) = err {
        //         return Err(err);
        //     }
        //     err
        // });

        if err.is_err() {
            eprintln!("The refactoring broke the code");
            return Err(RefactorStatusCodes::RefactoringProcucedBrokenCode as i32);
        }
        // TODO: output message / status that the code was broken after refactoring
    }

    if refactor_args.contains(&"--output-changes-as-json".to_owned()) {
        print!("{}", change_serialize::serialize_changes(changes)?);
    } else if refactor_args.contains(&"--output-replacements-as-json".to_owned()) {
        print!("{}", my_refactor_callbacks::serialize_file_replacements(&replacements)?);
    } else {
        print!("{}", content);
    }

    Ok(())
}

pub fn main() {
    rustc_driver::init_rustc_env_logger();
    rustc_driver::install_ice_hook();
    exit(
        run_rustc()
            .err()
            .unwrap_or(RefactorStatusCodes::Success as i32),
    )
}
