use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("close_over_variables", name).unwrap();
}

#[test]
#[ignore]
fn close_over_variables_pass_by_mut_ref() {
    run_test("pass_by_mut_ref");
}
#[test]
fn close_over_variables_pass_by_ref() {
    run_test("pass_by_ref");
}
#[test]
#[ignore]
fn close_over_variables_pass_by_val1() {
    run_test("pass_by_val1");
}
#[test]
#[ignore]
fn close_over_variables_pass_by_val2() {
    run_test("pass_by_val2");
}