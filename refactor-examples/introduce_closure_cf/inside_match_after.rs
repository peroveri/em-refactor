fn main() { }
fn foo() -> i32 {
    loop {
        match (|| {
            (0, None, Some(match 10 {
                11 => return (2, None, None),
                _ => return (3, Some(12), None)
            }))
        })() {
(2, _, _) => continue,
(3, a, _) => return a.unwrap(),
(_, _, a) => a.unwrap()}
    }
}