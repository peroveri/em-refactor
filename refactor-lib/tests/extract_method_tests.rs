extern crate assert_cmd;
use crate::test_case::run_testcase;

mod test_case;

#[test]
fn extract_method_owned_mut_value() {
    run_testcase("owned_mut_value").unwrap();
}
#[test]
fn extract_method_borrowed_mut_value() {
    run_testcase("borrowed_mut_value").unwrap();
}
#[test]
fn extract_method_owned_value() {
    run_testcase("owned_value").unwrap();
}
#[test]
fn extract_method_failure_borrow_used_later() {
    run_testcase("failure_borrow_used_later").unwrap();
}
#[test]
fn nested_block() {
    run_testcase("nested_block").unwrap();
}
#[test]
fn while_loop_inside() {
    run_testcase("while_loop_inside").unwrap();
}
#[test]
fn while_loop_outside() {
    run_testcase("while_loop_outside").unwrap();
}
#[test]
fn failure_selection_break_id() {
    run_testcase("failure_selection_break_id").unwrap();
}
#[test]
fn failure_selection_empty() {
    run_testcase("failure_selection_empty").unwrap();
}
#[test]
fn failure_selection_unbalanced() {
    run_testcase("failure_selection_unbalanced").unwrap();
}
#[test]
fn extract_method_breaks_code() {
    run_testcase("extract_method_breaks_code").unwrap();
}
#[test]
fn already_broken_code() {
    run_testcase("already_broken_code").unwrap();
}
/* Extract block */
#[test]
fn extract_block_1() {
    run_testcase("extract_block_1").unwrap();
}
#[test]
fn extract_block_with_expr() {
    run_testcase("extract_block_with_expr").unwrap();
}
