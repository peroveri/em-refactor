struct S;
fn main() {
    let mut s = S { };
    // extract start
    let mut t = &mut s;
    // extract end
    let u = &mut t;
}
