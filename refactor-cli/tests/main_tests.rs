extern crate assert_cli;

// #[test]
// fn calling_without_args_should_output_file_must_be_specified() {
//     assert_cli::Assert::main_binary()
//         .fails()
//         .and()
//         .stderr()
//         .is("Filename must be specified!")
//         .unwrap();
// }

#[test]
fn extract_method_01() {
    assert_cli::Assert::main_binary()
        .current_dir(std::path::Path::new("../refactor-examples/extract-method-01"))
        .with_args(&["--method extract-method --file main.rs --selection 3:0,3:11"])
        .succeeds()
        .and().stdout().is("")
        .and().stderr().is("")
        .unwrap();
}
