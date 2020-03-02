fn main() {
    let i = S;
    (|| {
        let _: S = i;
    })();
}
struct S;