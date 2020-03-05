fn main() {
    match (S {f: 0, g: 1}) {
        S {f, g: 1} => {let _: i32 = f;},
        S {f: 1, g: f} => {let _: i32 = f;},
        _ => {}
    }
}
struct S {f: i32, g: i32}
// in match arm body of line 3: f binds to either f or g
