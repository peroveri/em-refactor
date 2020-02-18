use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_block_vars", name).unwrap();
}

#[test]
fn extract_block_vars_1() {
    run_test("extract_block_1");
}
#[test]
fn extract_block_vars_field_used_later() {
    run_test("extract_block_field_used_later");
}
#[test]
fn extract_block_vars_multiple_use() {
    run_test("extract_block_multiple_use");
}
#[test]
fn extract_block_vars_mut() {
    run_test("extract_block_mut");
}