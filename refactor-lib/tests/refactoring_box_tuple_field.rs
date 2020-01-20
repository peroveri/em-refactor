use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("box_tuple_field", name).unwrap();
}

#[test]
fn box_field_field_access_2() {
    run_test("field_access_2");
}
#[test]
fn box_field_struct_definition_tuple() {
    run_test("struct_definition_tuple");
}
#[test]
fn box_field_struct_expression_tuple() {
    run_test("struct_expression_tuple");
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
fn box_field_visibility_tuple_pub_1() {
    run_test("visibility_tuple_pub_1");
}
#[test]
fn box_field_visibility_tuple_pub_crate() {
    run_test("visibility_tuple_pub_crate");
}