use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

pub const WORKSPACE_ARG: &str = "--workspace-root=../refactor-examples/crates/hello_world";
pub const WORKSPACE_ARG2: &str = "--workspace-root=../refactor-examples/crates/hello_world2";
pub const WORKSPACE_ARG_MULTI_ROOT: &str = "--workspace-root=../refactor-examples/crates/multi_root";
pub const WORKSPACE_ARG_MULTI_ROOT_OVERLAP: &str = "--workspace-root=../refactor-examples/crates/multi_root_overlap";
pub const WORKSPACE_DEPS_ARG: &str = "--workspace-root=../refactor-examples/crates/workspace_deps";
pub const WORKSPACE_NO_DEPS_ARG: &str = "--workspace-root=../refactor-examples/crates/workspace_no_deps";
pub const WORKSPACE_ARG_INVALID_CRATE: &str = "--workspace-root=../refactor-examples/crates/invalid_crate";

pub fn cargo_my_refactor() -> Command {
    Command::cargo_bin("cargo-my-refactor").unwrap()
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
