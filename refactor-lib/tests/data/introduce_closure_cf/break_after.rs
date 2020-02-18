fn main() {
    for i in 0..2 {
        match (|| {
            if i == 1 {
                return 1;
            }
            print!("{}", i);
            0
        })() {
            1 => break,
            _ => {}
        }
    }
}
// Extract the block at line 3 to line 8
// The conditional break at line 5 should be preserved.