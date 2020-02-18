use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_block_grammar", name).unwrap();
}

#[test]
fn extract_block_block_only_expr() {
    run_test("block_only_expr");
}
#[test]
#[ignore]
fn extract_block_block_with_expr_and_stmt_2() {
    run_test("block_with_expr_and_stmt_2");
}
#[test]
fn extract_block_block_with_expr_and_stmt_3() {
    run_test("block_with_expr_and_stmt_3");
}
#[test]
fn extract_block_block_with_expr_and_stmt_23() {
    run_test("block_with_expr_and_stmt_23");
}