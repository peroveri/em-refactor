struct S { field: Box<i32> }
fn main() {
    let s1 = S {field: Box::new(0)};
    println!("{}", (*s1.field));
}