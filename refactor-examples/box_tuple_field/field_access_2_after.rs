struct S(Box<T>);
struct T;
fn main() {
    let s1 = S (Box::new(T));
    let _: T = (*s1.0);
}