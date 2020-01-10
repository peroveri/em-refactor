struct S { field: i32 }
fn main() {
    match (S {field: 0}) {
        S {field: a @ 0} => {},
        _ => {}
    }
}
// Box field shouldn't be allowed here
// as we cannot write S {field: a @ Box::new(0)} or *a @ 0
// It could be supported by one of:
// - Use the experimental box syntax
// - Rewrite the @ to an if (is this always possible?)