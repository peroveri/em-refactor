use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_block_items", name).unwrap();
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