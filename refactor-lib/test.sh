export -n RUSTC_WRAPPER
cargo +nightly uninstall refactor_lib
#cargo my-refactor --method a --selection b --file c src/lib.rs
cargo +nightly install --path . --bin cargo-my-refactor
export RUSTC_WRAPPER='cargo-my-refactor'
cargo my-refactor
export -n RUSTC_WRAPPER