use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_block_vars", name).unwrap();
}

mod extract_block_vars {
    use super::*;
    
    #[test]
    fn expr() {
        run_test("expr");
    }
    #[test]
    fn extract_block_field_used_later() {
        run_test("extract_block_field_used_later");
    }
    #[test]
    fn preserve_mut() {
        run_test("preserve_mut");
    }
    #[test]
    fn redeclare_var() {
        run_test("redeclare_var");
    }
    #[test]
    fn redeclare_var_2() {
        run_test("redeclare_var_2");
    }
    #[test]
    fn statement() {
        run_test("statement");
    }
}