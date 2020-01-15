struct S;
fn main() {
    // extract start
    let s = S { };
    let t = &s;
    let u = &t;
    // extract end

    &u;
}
