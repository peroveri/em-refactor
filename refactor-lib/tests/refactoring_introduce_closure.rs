use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("introduce_closure", name).unwrap();
}

#[test]
#[ignore]
fn introduce_closure_assignment_single_variable() {
    run_test("assignment_single_variable");
}
#[test]
#[ignore]
fn introduce_closure_control_flow_break() {
    run_test("control_flow_break");
}
#[test]
#[ignore]
fn introduce_closure_mutate_single_variable() {
    run_test("mutate_single_variable");
}
