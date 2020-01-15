struct S;
fn main() {
    let s = S { };
    let t = &s;
    // extract start
    let u = &t;
    // extract end
    let v = &u;
}
