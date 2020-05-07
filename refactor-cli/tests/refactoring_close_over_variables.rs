use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("close_over_variables", name).unwrap();
}

mod close_over_variables {
    use super::*;
    
    #[test]
    #[ignore]
    fn borrow_from_trait() {
        run_test("borrow_from_trait");
    }
    #[test]
    fn multiple_params() {
        run_test("multiple_params");
    }
    #[test]
    fn pass_by_mut_ref() {
        run_test("pass_by_mut_ref");
    }
    #[test]
    fn pass_by_ref() {
        run_test("pass_by_ref");
    }
    #[test]
    fn pass_by_val1() {
        run_test("pass_by_val1");
    }
    #[test]
    #[ignore]
    fn self1() {
        run_test("self1");
    }
}