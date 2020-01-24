use crate::RefactorStatusCodes;
use std::path::Path;
use rustc_driver::{DefaultCallbacks, run_compiler};

/// Using Rerast's solution
/// https://github.com/google/rerast/blob/46dacd520f6bc63f4c37d9593b1b5163fc81611c/src/lib.rs
fn is_compiling_dependency(args: &[String]) -> bool {
    if let Some(path) = args.iter().find(|arg| arg.ends_with(".rs")) {
        Path::new(path).is_absolute()
    } else {
        false
    }
}

pub fn should_pass_to_rustc(rustc_args: &[String]) -> bool {
    return rustc_args.contains(&"--print=cfg".to_owned()) || is_compiling_dependency(&rustc_args);
}

pub fn pass_to_rustc(rustc_args: &[String]) -> Result<(), i32> {
    let mut default = DefaultCallbacks;
    let err = run_compiler(&rustc_args, &mut default, None, None); /* Some(Box::new(Vec::new())) */
    return if err.is_err() {
        Err(RefactorStatusCodes::RustcPassFailed as i32)
    } else {
        Ok(())
    };
}