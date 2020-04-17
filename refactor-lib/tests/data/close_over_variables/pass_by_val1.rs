fn main() {
    let i = S;
    (|| {
        let a: &S = &i;
        let b: S = i;
    })();
}
struct S;
// i is borrowed and consumed in the closure, but not used later