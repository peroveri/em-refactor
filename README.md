## [refactor-ls -- LSP client and server](./refactor-ls)
### How to run the language server locally
Requirements:
- Node.js - https://nodejs.org/en/
- Rust - https://www.rust-lang.org/tools/install
- Visual Studio Code - https://code.visualstudio.com

Steps:
- The rustc-dev component is required, and it can be added to the nightly toolchain using: ```rustup component add --toolchain nightly-2020-04-15 rustc-dev```
- Compile the refactoring tool by running ```cargo build --bins --release``` in the [./refactor-lib](./refactor-lib) folder.
- Run ```npm install``` in the [./refactor-ls](./refactor-ls) folder
- Open Visual Studio Code in the [./refactor-ls](./refactor-ls) folder: ```code refactor-ls```
- Run the build task (Ctrl+Shift+B)
- Debug the extension (Debug View -> Launch client)
- Configure settings
  - Open the vs code extension settings (File->Preferences->Settings) 
  - Set the "Refactoring Binary Path" setting to the absolute path of the binary file from step 2 ( it will be \<git repo folder> + /refactor-lib/target/release/cargo-my-refactor )
   

## [./refactor-lib/tests/data -- Examples in rust](./refactor-lib/tests/data)
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
