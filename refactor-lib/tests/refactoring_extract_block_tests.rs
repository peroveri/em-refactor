use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_block", name).unwrap();
}

#[test]
#[ignore]
fn extract_block_block_only_expr() {
    run_test("block_only_expr");
}
#[test]
#[ignore]
fn extract_block_block_with_expr_and_stmt_1() {
    run_test("block_with_expr_and_stmt_1");
}
#[test]
#[ignore]
fn extract_block_block_with_expr_and_stmt_2() {
    run_test("block_with_expr_and_stmt_2");
}
#[test]
#[ignore]
fn extract_block_borrow_field() {
    run_test("borrow_field");
}
#[test]
#[ignore]
fn extract_block_deref() {
    run_test("deref");
}
#[test]
fn extract_block_1() {
    run_test("extract_block_1");
}
#[test]
fn extract_block_field_used_later() {
    run_test("extract_block_field_used_later");
}
#[test]
fn extract_block_multiple_use() {
    run_test("extract_block_multiple_use");
}
#[test]
fn extract_block_mut() {
    run_test("extract_block_mut");
}
#[test]
fn extract_block_with_expr() {
    run_test("extract_block_with_expr");
}
#[test]
#[ignore]
fn extract_block_items_fn_declaration_1() {
    run_test("items_fn_declaration_1");
}
#[test]
#[ignore]
fn extract_block_items_struct_declaration() {
    run_test("items_struct_declaration");
}
#[test]
#[ignore]
fn extract_block_items_use_1() {
    run_test("items_use_1");
}
#[test]
#[ignore]
fn extract_block_items_use_2() {
    run_test("items_use_2");
}