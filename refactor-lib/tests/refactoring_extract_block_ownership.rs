use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_block_ownership", name).unwrap();
}

#[test]
fn extract_block_ownership_borrow_outlives() {
    run_test("borrow_outlives");
}
#[test]
fn extract_block_ownership_borrow_same_lifetime() {
    run_test("borrow_same_lifetime");
}