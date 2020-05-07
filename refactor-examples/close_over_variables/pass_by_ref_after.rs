fn main() {
    let i = S;
    (|i: &S| {
        let _: &S = &(*i);
    })(&i);
    i;
}
struct S;
// I is borrowed at l4 and consumed at l6.