struct S(i32);
pub fn main() {
    let mut s = S(0);
    s.0 = 1;
    s = S(2);
    println!("{}", s.0);
}
