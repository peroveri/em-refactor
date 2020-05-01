use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("pull_up_item_declarations", name).unwrap();
}

#[test]
fn pull_up_item_declarations_fn_decl() {
    run_test("fn_decl");
}
#[test]
fn pull_up_item_declarations_macro_no_decl() {
    run_test("macro_no_decl");
}
#[test]
fn pull_up_item_declarations_macro_with_decl() {
    run_test("macro_with_decl");
}
#[test]
fn pull_up_item_declarations_use_decl() {
    run_test("use_decl");
}
