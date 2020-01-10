struct S { field: Box<i32> }
fn main() {
    S { field: Box::new(1) };
}