fn main() {
    let mut i = S;
    let j = S;
    (|i: &mut crate::S, j: &crate::S| {
        (*i) = S;
        &(*j);
    })(&mut i, &j);
}
struct S;