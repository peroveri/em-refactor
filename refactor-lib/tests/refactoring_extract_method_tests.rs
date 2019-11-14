use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_method", name).unwrap();
}

#[test]
fn extract_method_owned_mut_value() {
    run_test("owned_mut_value");
}
// #[test]
// fn extract_method_borrowed_mut_value() {
//     run_test("borrowed_mut_value");
// }
// #[test]
// fn extract_method_owned_value() {
//     run_test("owned_value");
// }
// #[test]
// fn extract_method_failure_borrow_used_later() {
//     run_test("failure_borrow_used_later");
// }
#[test]
fn nested_block() {
    run_test("nested_block");
}
#[test]
fn while_loop_inside() {
    run_test("while_loop_inside");
}
#[test]
fn while_loop_outside() {
    run_test("while_loop_outside");
}
#[test]
fn failure_selection_break_id() {
    run_test("failure_selection_break_id");
}
#[test]
fn failure_selection_empty() {
    run_test("failure_selection_empty");
}
#[test]
fn failure_selection_unbalanced() {
    run_test("failure_selection_unbalanced");
}
#[test]
fn extract_method_breaks_code() {
    run_test("extract_method_breaks_code");
}
#[test]
fn already_broken_code() {
    run_test("already_broken_code");
}
