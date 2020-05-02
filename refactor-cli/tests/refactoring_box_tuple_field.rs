use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("box_tuple_field", name).unwrap();
}

mod box_tuple_field {
    use super::*;

    #[test]
    fn field_access_2() {
        run_test("field_access_2");
    }
    #[test]
    fn struct_definition_tuple() {
        run_test("struct_definition_tuple");
    }
    #[test]
    fn struct_expression_tuple() {
        run_test("struct_expression_tuple");
    }
    #[test]
    fn struct_pattern_tuple_1() {
        run_test("struct_pattern_tuple_1");
    }
    #[test]
    fn struct_pattern_tuple_id_pat() {
        run_test("struct_pattern_tuple_id_pat");
    }
    #[test]
    fn visibility_tuple_pub_1() {
        run_test("visibility_tuple_pub_1");
    }
    #[test]
    fn visibility_tuple_pub_crate() {
        run_test("visibility_tuple_pub_crate");
    }
}