fn main() {
    let i = S;
    (|| {
        let _: &S = &i;
    })();
    i;
}
struct S;