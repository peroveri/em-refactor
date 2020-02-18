use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("introduce_closure_grammar", name).unwrap();
}

#[test]
fn introduce_closure_grammar_assignment() {
    run_test("assignment");
}
#[test]
fn introduce_closure_grammar_statement() {
    run_test("statement");
}
