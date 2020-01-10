struct S { field: i32 }
fn main() {
    match (S {field: 0}) {
        S {field: a} if a == 0 => {},
        _ => {}
    }
}