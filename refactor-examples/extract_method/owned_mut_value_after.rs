struct S(i32);
fn inc(s: &mut S) {
s.0 = 1;
*s = S(2);
}
pub fn main() {
    let mut s = S(0);
    inc(&mut s);
    println!("{}", s.0);
}
