use crate::test_case::run_testcase;

mod test_case;

fn run_test(name: &str) {
    run_testcase("introduce_closure", name).unwrap();
}

// #[test]
// fn mut_() {
//     run_test("mut");
// }
#[test]
fn rhs() {
    run_test("rhs");
}
#[test]
fn stmt() {
    run_test("stmt");
}
