use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_block_ownership", name).unwrap();
}

mod extract_block_ownership {
    use super::*;
    
    #[test]
    fn borrow_field() {
        run_test("borrow_field"); // TODO: change error message assert
    }

    #[test]
    fn borrow_outlives() {
        run_test("borrow_outlives"); // TODO: change error message assert
    }
    #[test]
    fn borrow_same_lifetime() {
        run_test("borrow_same_lifetime");
    }
}