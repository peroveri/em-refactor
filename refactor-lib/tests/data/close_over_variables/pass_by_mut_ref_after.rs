fn main() {
    let mut i = S;
    (|i: &mut _| {
        (*i) = S;
    })(&mut i);
    i;
}
struct S;
// i is mutated at l4 and consumed at l6, so it must be passed in by mutable ref