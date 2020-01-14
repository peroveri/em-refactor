use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("box_field", name).unwrap();
}

#[test]
fn box_field_assignment_expression_1() {
    run_test("assignment_expression_1");
}
#[test]
fn box_field_assignment_expression_2() {
    run_test("assignment_expression_2");
}
#[test]
#[ignore]
fn box_field_attribute_macro_debug() {
    run_test("attribute_macro_debug");
}
#[test]
fn box_field_expression_macro() {
    run_test("expression_macro");
}
#[test]
fn box_field_field_access_1() {
    run_test("field_access_1");
}
#[test]
fn box_field_field_access_2() {
    run_test("field_access_2");
}
#[test]
fn box_field_field_access_call_expr_fn_generic() {
    run_test("field_access_call_expr_fn_generic");
}
#[test]
fn box_field_field_access_call_expr_fn_item() {
    run_test("field_access_call_expr_fn_item");
}
#[test]
fn box_field_field_access_call_expr_fn_trait() {
    run_test("field_access_call_expr_fn_trait");
}
#[test]
fn box_field_impl_self_param() {
    run_test("impl_self_param");
}
#[test]
fn box_field_impl_self_type() {
    run_test("impl_self_type");
}
#[test]
fn box_field_struct_definition_named_fields() {
    run_test("struct_definition_named_fields");
}
#[test]
fn box_field_struct_definition_tuple() {
    run_test("struct_definition_tuple");
}
#[test]
fn box_field_struct_expression_fields_1() {
    run_test("struct_expression_fields_1");
}
#[test]
#[ignore]
fn box_field_struct_expression_fields_and_overlap() {
    run_test("struct_expression_fields_and_overlap");
}
#[test]
#[ignore]
fn box_field_struct_expression_fields_and_self() {
    run_test("struct_expression_fields_and_self");
}
#[test]
fn box_field_struct_expression_fields_and_struct_base_1() {
    run_test("struct_expression_fields_and_struct_base_1");
}
#[test]
fn box_field_struct_expression_fields_and_struct_base_2() {
    run_test("struct_expression_fields_and_struct_base_2");
}
#[test]
fn box_field_struct_expression_fields_init_shorthand() {
    run_test("struct_expression_fields_init_shorthand");
}
#[test]
fn box_field_struct_expression_tuple() {
    run_test("struct_expression_tuple");
}
#[test]
fn box_field_struct_pattern_field_binding_1() {
    run_test("struct_pattern_field_binding_1");
}
#[test]
fn box_field_struct_pattern_field_binding_2() {
    run_test("struct_pattern_field_binding_2");
}
#[test]
fn box_field_struct_pattern_field_binding_at() {
    run_test("struct_pattern_field_binding_at");
}
#[test]
fn box_field_struct_pattern_field_binding_cond() {
    run_test("struct_pattern_field_binding_cond");
}
#[test]
fn box_field_struct_pattern_field_used() {
    run_test("struct_pattern_field_used");
}
#[test]
fn box_field_struct_pattern_tuple_1() {
    run_test("struct_pattern_tuple_1");
}
#[test]
fn box_field_struct_pattern_tuple_id_pat() {
    run_test("struct_pattern_tuple_id_pat");
}
#[test]
#[ignore]
fn box_field_trait_copy_cannot_contain_drop() {
    run_test("trait_copy_cannot_contain_drop");
}