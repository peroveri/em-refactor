fn main() {
    let mut s = S{f: 0};
    (|| {
        s.f = 0;
    })();
}
struct S {f: u32}