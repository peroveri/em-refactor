struct S<F>
    where F: Fn(T) -> U
{ field: Box<F> }
struct T;
struct U;
fn main() {
    let s1 = S {field: Box::new(&|_| {U}) };
    let _: U = (*(s1.field))(T);
}