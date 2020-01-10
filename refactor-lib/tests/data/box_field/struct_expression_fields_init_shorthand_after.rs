struct S { field: Box<i32> }
fn main() {
    let field = 1;
    S { field: Box::new(field) };
}