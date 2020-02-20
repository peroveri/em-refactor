use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("introduce_closure_cf", name).unwrap();
}

#[test]
fn introduce_closure_cf_all() {
    run_test("all");
}
#[test]
#[ignore]
fn introduce_closure_cf_break() {
    run_test("break");
}
#[test]
fn introduce_closure_loop_all() {
    run_test("loop_all");
}
#[test]
fn introduce_closure_cf_return_and_expr() {
    run_test("return_and_expr");
}