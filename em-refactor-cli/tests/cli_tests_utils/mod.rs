use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

pub const WORKSPACE_ARG: &str = "--workspace-root=../em-refactor-examples/crates/hello_world";
pub const WORKSPACE_ARG2: &str = "--workspace-root=../em-refactor-examples/crates/hello_world2";
pub const WORKSPACE_ARG_MULTI_ROOT: &str = "--workspace-root=../em-refactor-examples/crates/multi_root";
pub const WORKSPACE_ARG_MULTI_ROOT_OVERLAP: &str = "--workspace-root=../em-refactor-examples/crates/multi_root_overlap";
pub const WORKSPACE_DEPS_ARG: &str = "--workspace-root=../em-refactor-examples/crates/workspace_deps";
pub const WORKSPACE_NO_DEPS_ARG: &str = "--workspace-root=../em-refactor-examples/crates/workspace_no_deps";
pub const WORKSPACE_ARG_INVALID_CRATE: &str = "--workspace-root=../em-refactor-examples/crates/invalid_crate";

pub fn cargo_em_refactor() -> Command {
    Command::cargo_bin("cargo-em-refactor").unwrap()
}

pub fn create_tmp_dir() -> TempDir {
    let tmp_dir = TempDir::new().unwrap();
    let tmp_dir_path = tmp_dir.path();
    assert!(
        tmp_dir_path.is_dir(),
        "failed to create tmp dir: {}",
        tmp_dir_path.to_str().unwrap_or("")
    );
    tmp_dir
}
