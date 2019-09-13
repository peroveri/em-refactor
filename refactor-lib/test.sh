# export -n RUSTC_WRAPPER
# cargo +nightly uninstall refactor_lib
# #cargo my-refactor --method a --selection b --file c src/lib.rs
# cargo +nightly install --path . --bin cargo-my-refactor
# export RUSTC_WRAPPER='cargo-my-refactor'
# cargo my-refactor
# export -n RUSTC_WRAPPER
cargo clean && \
    cargo build && \
    cd examples/extract-method-01/project && \
    cargo clean && \
    cargo run --bin cargo-my-refactor --manifest-path /home/perove/dev/github.uio.no/refactor-rust/refactor-lib/Cargo.toml