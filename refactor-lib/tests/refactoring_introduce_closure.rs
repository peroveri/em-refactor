use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("introduce_closure", name).unwrap();
}

#[test]
fn introduce_closure_assignment_single_variable() {
    run_test("assignment_single_variable");
}
#[test]
fn introduce_closure_grammar_block_as_expression() {
    run_test("grammar_block_as_expression");
}
#[test]
fn introduce_closure_grammar_block_as_statement() {
    run_test("grammar_block_as_statement");
}
#[test]
fn introduce_closure_grammar_block_in_assignment() {
    run_test("grammar_block_in_assignment");
}
#[test]
fn introduce_closure_mutate_in_same_statement() {
    run_test("mutate_in_same_statement");
}
