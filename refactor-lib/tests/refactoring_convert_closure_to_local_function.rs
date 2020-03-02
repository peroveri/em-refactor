use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("convert_closure_to_local_function", name).unwrap();
}

#[test]
#[ignore]
fn convert_closure_to_local_function_no_annotations() {
    run_test("no_annotations");
}
#[test]
#[ignore]
fn convert_closure_to_local_function_returns_int() {
    run_test("returns_int");
}