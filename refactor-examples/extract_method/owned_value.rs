struct S(i32);
pub fn main() {
    let s = S(0);
    let t = s; // consume s
    let u = S(t.0 + 1);
    println!("{}", u.0);
}
