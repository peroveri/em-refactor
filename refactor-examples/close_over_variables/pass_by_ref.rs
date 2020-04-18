fn main() {
    let i = S;
    (|| {
        let _: &S = &i;
    })();
    i;
}
struct S;
// I is borrowed at l4 and consumed at l6.