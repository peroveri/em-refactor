struct S ( i32 );
fn main() {
    let s1 = S(0);
    match s1 {
        S(a @ 0) => {},
        S(_) => {},
    }
}