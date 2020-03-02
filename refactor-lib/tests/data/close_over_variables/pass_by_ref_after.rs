fn main() {
    let i = S;
    (|i: _| {
        let _: &S = (*&i);
    })(&i);
    i;
}
struct S;