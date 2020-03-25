use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("pull_up_item_declarations", name).unwrap();
}

#[test]
fn pull_up_item_declarations_example() {
    run_test("example");
}
