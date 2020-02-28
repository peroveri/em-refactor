fn main() {
    match (S{f: 0}) {
        S {f: f @ 0..=1} => {},
        _ => {f;}
    }
}
struct S{f: i32}