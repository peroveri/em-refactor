fn main() {
    let i = S;
    (|i: S| {
        let a: &S = &i;
        let b: S = i;
    })(i);
}
struct S;
// i is borrowed and consumed in the closure, but not used later