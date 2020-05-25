use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("inline_macro", name).unwrap();
}

mod inline_macro {
    use super::*;
    
    #[test]
    #[ignore]
    fn binding_preserve_local() {
        run_test("binding_preserve_local");
    }

    #[test]
    fn call_site_stmt_multiple() {
        run_test("call_site_stmt_multiple");
    }

    #[test]
    fn call_site_stmt_single() {
        run_test("call_site_stmt_single");
    }

    #[test]
    #[ignore]
    fn invocation_inside_macro() {
        run_test("invocation_inside_macro");
    }
}