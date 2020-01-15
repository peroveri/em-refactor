struct S;
fn main() {
    let s = S { };
    // extract start
    let t = &s;
    // extract end
    let u = &s;
}
