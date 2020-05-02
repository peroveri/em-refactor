use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("split_conflicting_match_arms", name).unwrap();
}

mod split_conflicting_match_arms {
    use super::*;
    
    #[test]
    #[ignore]
    fn if_let_expr() {
        run_test("if_let_expr");
    }
    #[test]
    #[ignore]
    fn match_expr() {
        run_test("match_expr");
    }
}