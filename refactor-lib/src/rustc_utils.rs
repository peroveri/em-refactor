use std::path::PathBuf;
use std::process::Command;

pub fn get_sys_root() -> String {
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
