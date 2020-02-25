use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("pull_up_item_declaration", name).unwrap();
}

#[test]
fn pull_up_item_declaration_fn_decl() {
    run_test("fn_decl");
}
#[test]
fn pull_up_item_declaration_use_decl() {
    run_test("use_decl");
}