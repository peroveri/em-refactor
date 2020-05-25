fn main() { }
fn foo(s1: S) {
    if let S {f, g: 1} | S {f: 1, g: f} = s1 {
        let _: i32 = f;
    }
}
struct S {f: i32, g: i32}
// in if let body at line 4: f binds to either S.f or S.g