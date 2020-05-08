fn main() {
    let mut s = S{f: 0};
    (|s: &mut S| {
        (*s).f = 0;
    })(&mut s);
}
struct S {f: u32}