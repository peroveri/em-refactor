fn main() {
    let i = S;
    (|| {
        let _: &S = &i;
        let _: S = i;
    })();
}
struct S;
// i is borrowed and consumed in the closure, but not used later