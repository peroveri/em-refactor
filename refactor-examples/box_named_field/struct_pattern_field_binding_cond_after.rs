struct S { field: Box<i32> }
fn main() {
    match (S {field: Box::new(0)}) {
        S {field: a} if (*a) == 0 => {},
        _ => {}
    }
}