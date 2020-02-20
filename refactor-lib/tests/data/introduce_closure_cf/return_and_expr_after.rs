fn main() {}
fn foo() -> i32 {
    for i in 10..12 {
        let j = 
        match (|| {
            if i == 11 {
                return (3, Some(20), None);
            }
            (0, None, Some(30))
        })() {
(3, a, _) => return a.unwrap(),
(_, _, a) => a.unwrap()};
    }
    return 0;
}
// Introduce closure at line 5 to 10
// The conditional return at line 7 should be preserved.