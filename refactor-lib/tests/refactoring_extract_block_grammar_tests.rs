use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_block_grammar", name).unwrap();
}

#[test]
fn extract_block_grammar_expr_and_stmt_2() {
    run_test("expr_and_stmt_2");
}
#[test]
fn extract_block_grammar_expr_and_stmt_23() {
    run_test("expr_and_stmt_23");
}
#[test]
fn extract_block_grammar_expr_and_stmt_3() {
    run_test("expr_and_stmt_3");
}
#[test]
fn extract_block_grammar_single_expr() {
    run_test("single_expr");
}