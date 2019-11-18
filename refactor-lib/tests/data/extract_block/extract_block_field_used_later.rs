struct S(i32);
fn main() {
    let s = S(0);
    let s = S(1);
    print!("{}", s.0);
}