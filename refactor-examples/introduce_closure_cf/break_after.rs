fn main() {
    for i in 10..12 {
        match (|| {
            if i == 11 {
                return (1, None);
            }
            print!("{}", i);
        (0, Some(()))})() {
(1, _) => break,
(_, a) => a.unwrap()}
    }
}
// Introduce closure at line 3 to 8
// The conditional break at line 5 should be preserved.