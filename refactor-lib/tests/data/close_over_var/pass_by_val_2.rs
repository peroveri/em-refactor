fn main() {
    let i = S;
    (|| {
        consume(i);
    })();
    &i;
}
fn consume(_: S) {}
struct S;