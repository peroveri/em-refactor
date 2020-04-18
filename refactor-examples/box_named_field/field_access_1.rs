struct S { field: T }
struct T {}
fn main() {
    let s1 = S { field: T {} };
    let _: T = s1.field;
}