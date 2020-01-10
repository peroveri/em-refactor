struct S { field: T }
struct T(U);
struct U();
fn main() {
    let mut s1 = S { field: T(U()) };
    s1.field = T(U());
    s1.field.0 = U();
}