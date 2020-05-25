use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("pull_up_item_declarations", name).unwrap();
}

mod pull_up_item_declarations {
    use super::*;
    
    #[test]
    fn fn_decl() {
        run_test("fn_decl");
    }
    #[test]
    fn macro_no_decl() {
        run_test("macro_no_decl");
    }
    #[test]
    fn macro_with_decl() {
        run_test("macro_with_decl");
    }
    #[test]
    fn use_decl() {
        run_test("use_decl");
    }
}