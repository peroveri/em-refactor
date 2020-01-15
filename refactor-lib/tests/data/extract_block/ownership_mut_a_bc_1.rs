struct S;
fn main() {
    let mut s = S { };
    // extract start
    let t = &mut s;
    // extract end
    let u = &mut s;
}
