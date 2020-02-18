use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("introduce_closure_cf", name).unwrap();
}

#[test]
#[ignore]
fn introduce_closure_control_flow_break() {
    run_test("control_flow_break");
}