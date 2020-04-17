fn main() {
    let mut i = S;
    let j = S;
    (|| {
        i = S;
        &j;
    })();
}
struct S;