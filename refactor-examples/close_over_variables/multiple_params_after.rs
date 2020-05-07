fn main() {
    let mut i = S;
    let j = S;
    (|i: &mut S, j: &S| {
        (*i) = S;
        &(*j);
    })(&mut i, &j);
}
struct S;