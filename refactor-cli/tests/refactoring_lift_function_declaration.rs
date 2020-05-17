use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("lift_function_declaration", name).unwrap();
}

mod lift_function_declaration {
    use super::*;
    
    #[test]
    #[ignore]
    fn method_mutable() {
        run_test("method_mutable");
    }
    #[test]
    #[ignore]
    fn method_parent_is_impl_for() {
        run_test("method_parent_is_impl_for");
    }
    #[test]
    #[ignore]
    fn method_parent_is_impl() {
        run_test("method_parent_is_impl");
    }
    #[test]
    #[ignore]
    fn parent_is_impl_for() {
        run_test("parent_is_impl_for");
    }
    #[test]
    fn parent_is_impl() {
        run_test("parent_is_impl");
    }
    #[test]
    fn parent_is_module() {
        run_test("parent_is_module");
    }
}