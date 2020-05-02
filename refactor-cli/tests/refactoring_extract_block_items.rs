use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_block_items", name).unwrap();
}

mod extract_block_items { // move to pull-up-item-decl
    use super::*;
    
    #[test]
    #[ignore]
    fn fn_decl() {
        run_test("items_fn_decl");
    }
    #[test]
    #[ignore]
    fn truct_declaration() {
        run_test("items_struct_declaration");
    }
    #[test]
    #[ignore]
    fn items_use_1() {
        run_test("items_use_1");
    }
    #[test]
    #[ignore]
    fn items_use_2() {
        run_test("items_use_2");
    }
}