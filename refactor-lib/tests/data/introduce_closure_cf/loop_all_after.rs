fn foo() -> i32 {
    let _: i32 =
    loop {
        let _: i32 = match (|| {
            if 1 == 1 {
                return (2, None, None, None);
            } else if 3 == 3 {
                return (1, Some(5), None, None);
            } else if 4 == 4 {
                return (3, None, Some(6), None);
            }
            (0, None, None, Some(10))
        })() {
(1, a, _, _) => break a.unwrap(),
(2, _, _, _) => continue,
(3, _, a, _) => return a.unwrap(),
(_, _, _, a) => a.unwrap()};

        continue;
    };
    0
}
fn main() { }
// Introduce closure at line 4 to 13