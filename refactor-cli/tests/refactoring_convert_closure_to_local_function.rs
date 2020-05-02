use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("convert_closure_to_local_function", name).unwrap();
}

mod convert_closure_to_local_function {
    use super::*;

    #[test]
    fn annotate_param() {
        run_test("annotate_param");
    }
    #[test]
    fn no_annotations() {
        run_test("no_annotations");
    }
    #[test]
    fn returns_int() {
        run_test("returns_int");
    }
}