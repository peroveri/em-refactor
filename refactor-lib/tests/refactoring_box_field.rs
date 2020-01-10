use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("box_field", name).unwrap();
}

#[test]
fn box_field_assignment_and_access_should_be_wrapped() {
    run_test("assignment_and_access_should_be_wrapped");
}
#[test]
fn box_field_match_as_ref() {
    run_test("match_as_ref");
}
#[test]
fn box_field_new_binding() {
    run_test("new_binding");
}
#[test]
fn box_field_struct_definition_named_fields() {
    run_test("struct_definition_named_fields");
}
#[test]
fn box_field_struct_definition_tuple() {
    run_test("struct_definition_tuple");
}