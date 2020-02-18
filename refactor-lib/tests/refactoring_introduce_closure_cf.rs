use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("introduce_closure_cf", name).unwrap();
}

#[test]
#[ignore]
fn introduce_closure_cf_break() {
    run_test("break");
}
#[test]
#[ignore]
fn introduce_closure_cf_return_and_expr() {
    run_test("return_and_expr");
}