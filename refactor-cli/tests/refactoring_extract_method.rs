use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_method", name).unwrap();
}

#[test]
#[ignore]
fn extract_method_example_1() {
    run_test("example_1");
}
