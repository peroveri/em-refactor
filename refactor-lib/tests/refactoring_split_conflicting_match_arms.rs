use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("split_conflicting_match_arms", name).unwrap();
}

#[test]
#[ignore]
fn split_conflicting_match_arms_if_let_expr() {
    run_test("if_let_expr");
}
#[test]
#[ignore]
fn split_conflicting_match_arms_match_expr() {
    run_test("match_expr");
}