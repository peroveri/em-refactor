struct S(Box<i32>);
fn main() {
    S(Box::new(1));
}