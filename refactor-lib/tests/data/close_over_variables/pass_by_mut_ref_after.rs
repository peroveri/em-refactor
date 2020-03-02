fn main() {
    let mut i = S;
    (|i: &mut _| {
        (*i) = S;
    })(&mut i);
    i;
}
struct S;