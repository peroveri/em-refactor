fn main() {
    let j = S;
    (|| {
        let i = j;
    })();
}
struct S;