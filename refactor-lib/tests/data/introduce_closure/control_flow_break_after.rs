fn main() {
    for i in 0..2 {
        let foo = || -> usize {
            if i == 1 {
                return 1;
            }
            print!("{}", i);
            0
        };
        match foo() {
            1 => break,
            _ => {}
        }
    }
}
// Extract the block at line 3 to line 8
// The conditional break at line 5 should be preserved.