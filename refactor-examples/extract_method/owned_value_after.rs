struct S(i32);
fn succ(s: S) -> S {
let t = s;
let u = S(t.0 + 1);
return u;
}
pub fn main() {
    let s = S(0);
    let u = succ(s);
    println!("{}", u.0);
}
