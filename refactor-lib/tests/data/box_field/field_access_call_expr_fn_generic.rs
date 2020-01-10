struct S<F>
    where F: Fn(T) -> U
{ field: F }
struct T;
struct U;
fn main() {
    let s1 = S {field: &|_| {U} };
    let _: U = (s1.field)(T);
}