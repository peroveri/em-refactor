use rustc::ty::TyCtxt;
use rustc_interface::{interface};
use rustc_span::{BytePos, Span};
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempdir::TempDir;

/**
 * Function that can be used to run unit tests.
 * Accepts a TokenStream (from quote) and a function with a single parameter TyCtxt.
 */
#[allow(unused)]
pub fn run_after_analysis<F>(program: quote::__rt::TokenStream, func: F)
where
    F: Fn(TyCtxt<'_>) -> (),
    F: Send,
{
    run_test_on_str(&format!("{}", program), func);
}

pub fn create_test_span(lo: u32, hi: u32) -> Span {
    Span::with_root_ctxt(BytePos(lo), BytePos(hi))
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

struct RustcTestCallbacks<F>(F);

impl<F> rustc_driver::Callbacks for RustcTestCallbacks<F>
where
    F: Fn(TyCtxt<'_>) -> (),
    F: Send,
{
    // fn after_expansion(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
    //     rustc_driver::Compilation::Continue
    // }
    fn after_analysis<'tcx>(
        &mut self, 
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        queries
            .global_ctxt()
            .unwrap()
            .peek_mut()
            .enter(|tcx| self.0(tcx));
        rustc_driver::Compilation::Stop
    }
}

fn run_test_on_str<F>(program: &str, func: F)
where
    F: Fn(TyCtxt<'_>) -> (),
    F: Send,
{
    let tmp_dir =
        TempDir::new("my_refactoring_tool").unwrap_or_else(|_| panic!("failed to create tmp dir"));
    let tmp_dir_path = tmp_dir.path();
    set_main_rs(tmp_dir_path, program).unwrap_or_else(|_| panic!("failed to set main rs"));

    let rustc_args = [
        ".".to_owned(),
        "--sysroot".to_owned(),
        get_sys_root(),
        tmp_dir_path.join("main.rs").to_str().unwrap().to_string(),
        "--allow".to_owned(),
        "dead_code".to_owned(),
        "--allow".to_owned(),
        "deprecated".to_owned(),
        "--allow".to_owned(),
        "unused".to_owned(),
        "--crate-type".to_owned(),
        "lib".to_owned(),
        format!("--out-dir={}", tmp_dir_path.to_str().unwrap()),
    ];
    let mut callbacks = RustcTestCallbacks(func);

    // rustc_driver::init_rustc_env_logger();
    // rustc_driver::install_ice_hook();

    let err = rustc_driver::run_compiler(&rustc_args, &mut callbacks, None, None);
    err.unwrap();
}
fn set_main_rs(path: &Path, content: &str) -> std::io::Result<()> {
    let path = path.join("./main.rs");
    assert!(
        !path.exists(),
        "main.rs already existed: {}",
        path.to_str().unwrap()
    );
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
