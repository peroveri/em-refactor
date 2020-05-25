use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("box_named_field", name).unwrap();
}

mod box_named_field {
    use super::*;
    #[test]
    fn assignment_expression_1() {
        run_test("assignment_expression_1");
    }
    #[test]
    fn assignment_expression_2() {
        run_test("assignment_expression_2");
    }
    #[test]
    fn attribute_macro_debug() {
        run_test("attribute_macro_debug");
    }
    #[test]
    fn expression_macro() {
        run_test("expression_macro");
    }
    #[test]
    fn field_access_1() {
        run_test("field_access_1");
    }
    #[test]
    fn field_access_call_expr_fn_generic() {
        run_test("field_access_call_expr_fn_generic");
    }
    #[test]
    fn field_access_call_expr_fn_item() {
        run_test("field_access_call_expr_fn_item");
    }
    #[test]
    fn field_access_call_expr_fn_trait() {
        run_test("field_access_call_expr_fn_trait");
    }
    #[test]
    fn impl_self_param() {
        run_test("impl_self_param");
    }
    #[test]
    fn impl_self_type() {
        run_test("impl_self_type");
    }
    #[test]
    #[ignore]
    fn macro_arg_used_twice() {
        run_test("macro_arg_used_twice");
    }
    #[test]
    #[ignore]
    fn macro_rule_used_twice() {
        run_test("macro_rule_used_twice");
    }
    #[test]
    fn struct_definition_named_fields() {
        run_test("struct_definition_named_fields");
    }
    #[test]
    fn struct_expression_fields_1() {
        run_test("struct_expression_fields_1");
    }
    #[test]
    #[ignore]
    fn struct_expression_fields_and_overlap() {
        run_test("struct_expression_fields_and_overlap");
    }
    #[test]
    #[ignore]
    fn struct_expression_fields_and_self() {
        run_test("struct_expression_fields_and_self");
    }
    #[test]
    fn struct_expression_fields_and_struct_base_1() {
        run_test("struct_expression_fields_and_struct_base_1");
    }
    #[test]
    fn struct_expression_fields_and_struct_base_2() {
        run_test("struct_expression_fields_and_struct_base_2");
    }
    #[test]
    fn struct_expression_fields_init_shorthand() {
        run_test("struct_expression_fields_init_shorthand");
    }
    #[test]
    fn struct_pattern_field_binding_1() {
        run_test("struct_pattern_field_binding_1");
    }
    #[test]
    fn struct_pattern_field_binding_2() {
        run_test("struct_pattern_field_binding_2");
    }
    #[test]
    fn struct_pattern_field_binding_at() {
        run_test("struct_pattern_field_binding_at");
    }
    #[test]
    fn struct_pattern_field_binding_cond() {
        run_test("struct_pattern_field_binding_cond");
    }
    #[test]
    fn struct_pattern_field_used() {
        run_test("struct_pattern_field_used");
    }
    #[test]
    #[ignore]
    fn trait_copy_cannot_contain_drop() {
        run_test("trait_copy_cannot_contain_drop");
    }
    #[test]
    fn visibility_named_pub_1() {
        run_test("visibility_named_pub_1");
    }
    #[test]
    fn visibility_named_pub_crate() {
        run_test("visibility_named_pub_crate");
    }
}