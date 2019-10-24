use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("box_field", name).unwrap();
}

#[test]
fn assignment_and_access_should_be_wrapped() {
    run_test("assignment_and_access_should_be_wrapped");
}
#[test]
fn field_should_be_boxed() {
    run_test("field_should_be_boxed");
}
#[test]
fn match_as_ref() {
    run_test("match_as_ref");
}
#[test]
fn tuple_struct_field_should_be_boxed() {
    run_test("tuple_struct_field_should_be_boxed");
}