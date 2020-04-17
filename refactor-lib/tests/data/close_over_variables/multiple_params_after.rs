fn main() {
    let mut i = S;
    let j = S;
    (|i: &mut _, j: _| {
        (*i) = S;
        (*&j);
    })(&mut i, &j);
}
struct S;