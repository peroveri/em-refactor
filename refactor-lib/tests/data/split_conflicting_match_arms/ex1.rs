fn main() {
    match (S {f: 0, g: 1}) {
        S {f, g: 1} | S {f: 1, g: f} => {f;g;},
        _ => {}
    }
}
struct S {f: i32, g: i32}
// in match arm body of line 3: f binds to either f or g