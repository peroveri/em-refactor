struct S(T);
struct T;
fn main() {
    let s1 = S (T);
    let _: T = s1.0;
}