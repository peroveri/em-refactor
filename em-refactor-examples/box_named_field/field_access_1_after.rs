struct S { field: Box<T> }
struct T {}
fn main() {
    let s1 = S { field: Box::new(T {}) };
    let _: T = (*s1.field);
}