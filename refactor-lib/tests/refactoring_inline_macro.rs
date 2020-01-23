use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("inline_macro", name).unwrap();
}

#[test]
fn inline_macro_binding_preserve_local() {
    run_test("binding_preserve_local");
}

#[test]
fn inline_macro_call_site_stmt_multiple() {
    run_test("call_site_stmt_multiple");
}

#[test]
fn inline_macro_call_site_stmt_single() {
    run_test("call_site_stmt_single");
}

#[test]
fn inline_macro_invocation_inside_macro() {
    run_test("invocation_inside_macro");
}
