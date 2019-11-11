use rustc::ty::TyCtxt;
use rustc_interface::interface;
use std::env::{current_dir, set_current_dir};
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use syntax_pos::{BytePos, Span};

/**
 * Function that can be used to run unit tests. 
 * Accepts a TokenStream (from quote) and a function with a single parameter TyCtxt.
 */
#[allow(unused)]
pub fn run_test<F>(program: quote::__rt::TokenStream, func: F)
where
    F: Fn(TyCtxt<'_>) -> (),
    F: Send,
{
    run_test_on_str(&format!("{}", program), func);
}

pub fn create_test_span(lo: u32, hi: u32) -> syntax_pos::Span {
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
    fn after_analysis(&mut self, compiler: &interface::Compiler) -> rustc_driver::Compilation {
        compiler.session().abort_if_errors();
        compiler
            .global_ctxt()
            .unwrap()
            .peek_mut()
            .enter(|tcx| self.0(tcx));
        rustc_driver::Compilation::Stop
    }
}

fn change_dir() -> std::io::Result<()> {
    set_current_dir(current_dir()?.join("../tmp"))?;
    Ok(())
}

fn run_test_on_str<F>(program: &str, func: F)
where
    F: Fn(TyCtxt<'_>) -> (),
    F: Send,
{
    change_dir().unwrap();
    set_main_rs(program).unwrap();

    let rustc_args = [
        ".".to_owned(),
        "--sysroot".to_owned(),
        get_sys_root(),
        "unit_test.rs".to_owned(),
        "--allow".to_owned(),
        "dead_code".to_owned(),
        "--allow".to_owned(),
        "deprecated".to_owned(),
        "--allow".to_owned(),
        "unused".to_owned(),
        "--crate-type".to_owned(),
        "lib".to_owned(),
    ];
    let mut callbacks = RustcTestCallbacks(func);

    // rustc_driver::init_rustc_env_logger();
    // rustc_driver::install_ice_hook();

    let err = rustc_driver::run_compiler(&rustc_args, &mut callbacks, None, None);
    err.unwrap();
}
fn set_main_rs(content: &str) -> std::io::Result<()> {
    let path = Path::new("./unit_test.rs");
    if !path.is_file() {
        panic!("file didnt exist: {}", path.to_str().unwrap());
    }
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
