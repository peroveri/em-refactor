fn foo() -> i32 {
    for i in 10..12 {
        let _: i32 = match (|| {
            if i == 1 {
                return (2, None, None);
            } else if i == 3 {
                return (1, None, None);
            } else if i == 4 {
                return (3, Some(1), None);
            }
            print!("{}", i);
            (0, None, Some(10))
        })() {
(1, _, _) => break,
(2, _, _) => continue,
(3, a, _) => return a.unwrap(),
(_, _, a) => a.unwrap()};
    }
    0
}
fn main() { }
// Introduce closure at line 3 to 13