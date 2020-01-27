mod type_lookup;

use super::{arg_value};

pub fn should_provide_type(refactor_args: &[String]) -> bool {
    return refactor_args.contains(&"--provide-type".to_owned());
}

pub fn provide_type(refactor_args: &[String], rustc_args: &[String]) -> Result<(), i32> {
    let selection = arg_value(refactor_args, "--selection", |_| true).unwrap();
    let file_name = arg_value(refactor_args, "--file", |_| true).unwrap();
    if let Ok(()) = type_lookup::provide_type(rustc_args, file_name, selection) {
        return Ok(());
    } else {
        return Err(-1);
    }
}