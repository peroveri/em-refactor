use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("extract_method", name).unwrap();
}

mod extract_method {
    use super::*;
    
    #[test]
    fn eexample_1() {
        run_test("example_1");
    }
}