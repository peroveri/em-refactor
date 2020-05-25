struct S { field: i32 }
fn main() {
    match (S {field: 0}) {
        S {field: 0} => {},
        _ => {}
    }
}