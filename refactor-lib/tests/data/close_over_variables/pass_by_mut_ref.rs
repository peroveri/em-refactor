fn main() {
    let mut i = S;
    (|| {
        i = S;
    })();
    i;
}
struct S;