fn main() {
    match (S{f: 0}) {
        S {f: f} if match f {
            0..=1 => true, _ => false
        } => {f;},
        _ => {}
    }
}
struct S{f: i32}