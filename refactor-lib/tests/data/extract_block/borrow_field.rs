struct S {
    t: T,
}
struct T(i32);
fn main() {
    let s = S { t: T(0) };
    let t = &s.t;

    println!("{}, {}", &s.t.0, &t.0);
}
/* 
 * Here we extract line 6 and 7. The issue is that 's' will be moved afterwards while 't' is borrowed.
 * For this particular case, it could be solved by introducing a borrow on line 6.
 */
