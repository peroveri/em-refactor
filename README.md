# Running from the terminal
## Requirements
- Rust - https://www.rust-lang.org/tools/install

## Steps
1. The rustc-dev component is required for compiling this repo, and it can be added to the nightly toolchain using:

```sh
rustup component add --toolchain nightly-2020-04-15 rustc-dev
```

2. Compile the refactoring tool by running the following command in the root folder of this repository:

```sh
cargo build --bins --release
```

3. Candidates and refactorings can be run either in the current directory, or by setting the --target-dir="path" option

Candidates:

```cargo-my-refactor candidates <box-field/extract-method> [--target-dir=PATH]```
```sh
./target/release/cargo-my-refactor candidates box-field --target-dir="../path/to/project"
./target/release/cargo-my-refactor candidates extract-method
```

Refactoring:

```cargo-my-refactor refactor <box-field/extract-block/extract-method/...> <FILE> <SELECTION> [--target-dir=PATH]```
```sh
./target/release/cargo-my-refactor refactor box-field refactor-lib/src/refactorings/visitors/struct_field_access_expression_collector.rs 1242:1255
```

# Running in VS Code

## Requirements
- Run Step 1 & 2 from [Running from the terminal](#running-from-the-terminal)
- Node.js - https://nodejs.org/en/
- Visual Studio Code - https://code.visualstudio.com

## Steps
- Run ```npm install``` in the [./refactor-ls](./refactor-ls) folder
- Open Visual Studio Code in the [./refactor-ls](./refactor-ls) folder: ```code refactor-ls```
- Run the build task (Ctrl+Shift+B)
- Debug the extension (Debug View -> Launch client)
- Configure settings in the new window that launched
  - Open the vs code extension settings (File->Preferences->Settings) 
  - Set the "Refactoring Binary Path" setting to the absolute path of the binary file from step 2 ( it will be \<git repo folder> + /target/release/cargo-my-refactor )

# [./refactor-examples -- Examples in rust](./refactor-examples)
This project should contain a list of valid refactorings that can be used for unit tests.

Each refactoring example should contain the code before, after and the arguments passed to the refactoring tool. The arguments must contain refactoring definition name (e.g. 'extract-method') and selection (from and to). Other arguments might be required depending on the actual refactoring.

So for the testcase 'extract-method-01', the following files are required
* `extract-method-01.rs`
    * The code before refactoring
* `extract-method-01_after.rs`
    * The expected code after refactoring
* `extract-method-01.json`
    * The arguments to be passed
    
If the test is expected to fail, then the file ending with `_after.rs` is not required.

TODO: When searching automatically for candidates that can be refactored, the arguments should not be required. How can that be tested automatically?

## [./refactor-lib -- Refactor library](./refactor-lib)
The project containing the actual refactorings. 

There are two executable files, one from main.rs and one from driver.rs.

main.rs calls ```cargo check``` with the executable from driver.rs as argument. The RUSTC_WRAPPER flag is set to 1 when calling cargo. Cargo then calls the executable with the same argument as if it was calling rustc. We can then pass callbacks when we run rustc_driver::run_compiler.

The tool can be invoked like this:

`cargo run --bin my-refactor-driver refactor-examples/extract_method/owned_mut_value.rs  -- --refactoring=extract-block --selection=39:46`
