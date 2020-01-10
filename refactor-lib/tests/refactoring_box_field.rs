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
fn box_field_field_access_1() {
    run_test("field_access_1");
}
#[test]
#[ignore]
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
fn box_field_struct_expression_fields_and_struct_base_1() {
    run_test("struct_expression_fields_and_struct_base_1");
}
#[test]
fn box_field_struct_expression_fields_and_struct_base_2() {
    run_test("struct_expression_fields_and_struct_base_2");
}
#[test]
#[ignore]
fn box_field_struct_expression_fields_init_shorthand() {
    run_test("struct_expression_fields_init_shorthand");
}
#[test]
#[ignore]
fn box_field_struct_expression_tuple() {
    run_test("struct_expression_tuple");
}
#[test]
fn box_field_struct_pattern_field_binding_1() {
    run_test("struct_pattern_field_binding_1");
}
#[test]
fn box_field_struct_pattern_field_binding_cond() {
    run_test("struct_pattern_field_binding_cond");
}
#[test]
fn struct_pattern_field_used() {
    run_test("struct_pattern_field_used");
}