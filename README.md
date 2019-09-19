## [./refactor-examples -- Examples in rust](./refactor-examples)
This project should contain a list of valid refactorings that can be used for unit tests.

Each refactoring example should contain the code before, after and the arguments passed to the refactoring tool. The arguments must contain refactoring definition name (e.g. 'extract-method') and selection (from and to). Other arguments might be required depending on the actual refactoring.

So for the testcase 'extract-method-01', the following files are required
* `extract-method-01.rs`
    * The code before refactoring
* `extract-method-01.after.rs`
    * The expected code after refactoring
* `extract-method-01.json`
    * The arguments to be passed

When searching automatically for candidates that can be refactored, the arguments should not be required. How can that be tested automatically?

## [./refactor-lib -- Refactor library](./refactor-lib)
The project containing the actual refactorings. 

### Rustc callbacks

The way the term 'lint' is used here seems to be fixing (broken) code and refactoring, so it is some kind of issue with the source code that the tool points out and maybe it also suggest a 

* Register EarlyLintPass/LateLintPass
    * Register the 'lints' in rustc_driver::Callbacks::after_parsing
    * EarlyLintPass is with original syntax, but before type checking
    * LateLintPass is after type checking, but with a desugared syntax
    * Similar to rustc builtin [lints](https://rust-lang.github.io/rustc-guide/diagnostics.html#lints) and clippy lints
    * Output the suggested refactoring with --error-format json (code change + message),
    which may then be applied later by another tool
    * May be some suggestions here: [github clippy issues](https://github.com/rust-lang/rust-clippy/issues?q=is%3Aissue+is%3Aopen)
* rustc_driver::Callbacks::after_parsing/after_analysis
    * Similar to [rerast](https://github.com/google/rerast/), a search and replace tool.
    * Handle the file changes internally

## [refactor-cli -- CLI](./refactor-cli)


## [refactor-ls -- LSP client and server](./refactor-ls)
TODO: add LSP client and server. Could use the example at https://github.com/Microsoft/vscode-extension-samples/tree/master/lsp-sample as a starting point.
