struct S(i32);
fn main() {
    let s = S(0);
    let s0 = {
    let s0 = &s.0;
s0
};
    let t = *s0;

    println!("{}", &t);
}
