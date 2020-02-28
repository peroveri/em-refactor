fn main() {
    let j = S;
    (|j: S| {
        let i = j;
    })(j);
}
struct S;