use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("introduce_closure_cf", name).unwrap();
}

mod introduce_closure {
    use super::*;
    
    #[test]
    fn all() {
        run_test("all");
    }
    #[test]
    fn r#break() {
        run_test("break");
    }
    #[test]
    fn inside_match() {
        run_test("inside_match");
    }
    #[test]
    fn loop_all() {
        run_test("loop_all");
    }
    #[test]
    fn outside() {
        run_test("outside");
    }
    #[test]
    fn return_and_expr() {
        run_test("return_and_expr");
    }
}