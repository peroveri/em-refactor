fn main() {
    let i = S;
    (|i: _| {
        let _: S = i;
    })(i);
}
struct S;