extern crate assert_cmd;
extern crate my_refactor_lib;

use std::process::Command;
use assert_cmd::prelude::*;
use my_refactor_lib::Change;

#[test]
fn extract_method_owned_mut_value() {
    // using cargo check to invoke the refactoring tool, so we need to force it to check files when they have not been changed (instead of running cargo clean before)
    Command::new("touch").arg("-c").arg("../refactor-examples/extract_method_01/src/main.rs").unwrap().assert().success();
    Command::cargo_bin("cargo-my-refactor")
        .unwrap()
        .current_dir("../refactor-examples/extract_method_01")
        .arg("--")
        .arg("--")
        .arg("--refactoring=extract-method")
        .arg("--file=src/owned_mut_value.rs")
        .arg("--selection=43:53")
        .arg("--new_function=inc")
        // add --args-from-file=...
        // add --print-changed-files
        .assert()
        .success()
        //stdout=read_file("owned_mut_value_after.rs")
        .stdout(format!("{:?}\n{:?}\n", Change {
            file_name: "src/owned_mut_value.rs".to_owned(),
            start: 0,
            end: 0,
            replacement: "fn inc(i: &mut i32) {\ni += 1;\n}".to_owned()
        }, Change {
            file_name: "src/owned_mut_value.rs".to_owned(),
            start: 0,
            end: 0,
            replacement: "inc(&mut i);".to_owned()
        }));
        // .stderr("");
}
