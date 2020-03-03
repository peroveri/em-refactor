fn main() {
    let i = S;
    (|| {
        let _: &S = &i;
    })();
}
struct S;
// i is borrowed in the closure, but not used later