fn main() {
    let i = S;
    (|i: _| {
        let _: S = i;
    })(i);
}
struct S;
// i is borrowed and consumed in the closure, but not used later